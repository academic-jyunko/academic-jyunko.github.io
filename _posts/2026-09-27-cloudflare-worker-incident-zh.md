---
layout: post
title: "从一次部署失败到全站脚本注入：Cortex 的 Cloudflare 事件复盘"
date: 2026-09-27 12:10:00 +0800
author: HsiangNianian
lang: zh-CN
permalink: /2026/09/27/cloudflare-worker-incident-zh.html
toc: true
excerpt: "一行 pnpm 错误把我带回构建系统，一段不属于项目的 HTML 又把排查带到了 Cloudflare 账户。本文依据构建日志、Worker 源码与审计记录，重建一次边缘脚本注入的发现与处置过程。"
description: "Cortex 的 Cloudflare Workers 注入事件复盘：部署故障、全站路由、账户令牌、时间线、证据边界与恢复验证。"
image: /assets/incidents/2026-09-27-cloudflare/request-path-zh@2x.png
---

[English version](/2026/09/27/cloudflare-worker-incident-en.html)

2026 年 9 月 27 日，我最初只是想修好一次部署。Cortex 的私有开发分支已经推到 GitHub，Cloudflare 却在安装依赖时退出。错误信息看起来很熟悉：锁文件和补丁配置不一致。

修复构建之后，检查没有停在绿色的部署状态上。浏览器收到的页面里出现了与项目毫无关系的 Betaflight 推广内容，甚至多出一个隐藏的一级标题。搜索仓库，找不到；搜索构建产物，也找不到。沿着这段多出来的 HTML 往回查，最后找到的是 Cloudflare 账户里的另一个 Worker，以及覆盖整个域名的路由。

这篇文章记录这次排查。它是一份基于现场证据的个案报告：可以确认什么、如何确认，以及哪些问题到写作时仍没有答案。文中时间统一为 **Asia/Shanghai，UTC+8**；原始日志里的 `Z` 时间已换算。构建故障与注入事件分别有证据支持，**目前没有证据表明 pnpm 故障造成了这次攻击**。

## 1. 起点：工具探测信息与真正执行的版本不一致

最早的失败发生在 09:00 左右：

```text
ERR_PNPM_LOCKFILE_CONFIG_MISMATCH
The current "patchedDependencies" configuration doesn't match
the value found in the lockfile
```

项目使用 OpenNext 将应用部署到 Cloudflare，并通过 pnpm 管理一份兼容性补丁。单看报错，很容易把问题归到“修改补丁之后忘记更新锁文件”。但本地同一份代码可以完成冻结安装，这个解释还不够。

排查时，我们把检查放进了 Cloudflare 实际执行的构建命令：打印补丁文件的哈希、锁文件中的补丁记录，再运行 `pnpm --version`。诊断日志出现了一个值得记下的差异：

```text
环境探测：pnpm@10.11.1
实际命令：pnpm --version
实际输出：9.10.0
```

在这次构建环境中，探测到的版本并不等于命令最终调用的版本。补丁内容及其锁文件记录没有发生预期之外的变化；固定使用 pnpm 10.33.2 后，同样的冻结安装通过了。修复保留了 `--frozen-lockfile`，并在仓库和 Cloudflare 构建变量中明确约定 pnpm、Node 版本。这里能确认的是版本选择与补丁锁兼容性问题；仅凭这些日志，还不能确定旧 pnpm 为什么在该环境中被选中。

随后，本地手动发布和 GitHub 推送触发的发布都走通了。自动流程依次执行检查、测试、构建、发布。过程中还清理了 OpenNext 产物可能带入的本地环境文件，运行时密钥改由 Cloudflare Secrets 提供。这是部署边界的额外收紧，不能倒过来当作“攻击者从这些文件取得了凭据”的证据。

如果验收只看构建结果，故事会在这里结束。

## 2. 页面正常打开，多出来的内容也确实存在

部署后的检查发现，应用能正常展示，但返回内容中夹带了隐藏的推广文本和链接，目标域名为 `betaflight[.]uk[.]com`。这个名称出现在注入脚本里；本文不据此推断 Betaflight 开源项目与攻击有关。

接下来的比较围绕同一组特征展开：推广域名、`content-extra` 类名，以及 `__contentLoaded` 标记。

| 检查对象 | 这组注入特征 |
| --- | --- |
| 仓库源码及本地构建产物 | 未检出 |
| Cloudflare 上已部署的 Cortex Worker 代码 | 未检出 |
| 自定义域名返回的 HTML | 检出 |
| 账户中的另一份 Worker 源码 | 检出对应注入实现 |

“未检出”只针对这些已知特征，不等于给整份代码做了安全证明。但这组差异足以改变排查方向：Cortex 生成的内容到达浏览器之前，还经过了别的可执行环节。

证书也没有给出明显异常。访问仍走 HTTPS，TLS 校验正常。这一点后来很好理解：页面是在合法 Cloudflare 账户控制的请求链路中被改写的，浏览器看到的证书无需因此失效。

## 3. 注入位置：站在应用前面的另一个 Worker

账户里有一个名为 `cf-w-d6b620c5` 的 Worker。它不属于 Cortex 的部署配置，挂载的路由却是：

```text
*hydroroll.team/*
```

这个模式把影响范围从单个应用扩展到了整个域名下可能匹配的请求。实际影响仍取决于请求路径、路由优先级和代码分支，不能把路由范围直接换算成受害访客数量。

Cloudflare 的路由 Worker 可以运行在 Custom Domain Worker 前面；前者调用 `fetch(request)` 时，可以继续取得后者的响应。这是平台公开支持的组合方式。[1] 在这次事件中，额外的 Worker 利用了这个位置：先取得应用响应，再用 `HTMLRewriter` 向 HTML 追加脚本。

<figure>
  <a href="/assets/incidents/2026-09-27-cloudflare/request-path-zh.svg"><img src="/assets/incidents/2026-09-27-cloudflare/request-path-zh.svg" alt="浏览器先访问恶意路由 Worker，后者取得 Cortex 的 HTML 后追加脚本，再把修改后的页面返回浏览器" width="960" height="820" loading="lazy" style="width:100%;height:auto;"></a>
  <figcaption>图 1．依据保全的源码和路由配置重建的请求链路。它解释了为什么重新发布 Cortex 本身不能移除注入。点击图片可查看原始 SVG，也可<a href="/assets/incidents/2026-09-27-cloudflare/request-path-zh@2x.png">下载 PNG</a>。</figcaption>
</figure>

因此，Git 仓库和应用 Worker 都可以保持没有上述注入特征，而最终页面仍被污染。只重新部署应用，不会顺带删除账户里独立存在的路由 Worker。

## 4. 源码显示的能力，比隐藏链接更多

保全这份 Worker 后，我们对下载的 Worker 做静态阅读和字符串解码，没有在本地重放它，也没有主动获取或运行条件投放分支引用的外部载荷。前述隐藏内容则是在实际页面中观察到的。源码中有几类不同的行为：

- **隐藏内容注入。** 脚本向页面插入推广链接、标题和文本，通过离屏定位、透明度等样式隐藏。这一部分与现场返回内容相符。
- **条件式脚本投放。** 另一条分支依据 Windows User-Agent、Cookie、网络组织信息，以及对代理或托管网络的判断，决定是否获取并追加外部脚本。外部地址使用了简单的 XOR 字符串混淆。
- **特殊路径代理。** 以 `/_r/` 开头的请求可被转发到外部端点；实现会带上原请求的部分上下文，并附加客户端 IP、国家和城市信息。
- **命令式响应分支。** 源码还包含针对特定 User-Agent 追加 PowerShell 命令的逻辑。它受前置分支约束；看到这段代码，并不意味着这次事件中有人实际执行了该命令。

解码得到的脚本来源包含 `macrium[.]info`。这里的域名均做了断链书写，不应把它们当作访问建议。公开保留的样本指纹为：

```text
SHA-256（保全的 Worker 源码文件）
2659f437dbf42d308d963fd08b9ba7f90ecd8544d403c8deec650bc35f55cc47
```

还有一个细节解释了为什么部分检查容易漏掉它：源码对 `/api`、`/_next`、`robots`、`sitemap` 以及多种静态资源路径直接放行；部分终端只得到隐藏推广内容，不进入额外载荷分支。API 返回 200，或者某一台电脑没有看到弹窗，都不足以证明 HTML 没有被改写。

这些是源码能支持的**行为能力**。我们没有完整的访问日志或终端取证，不能据此宣称已经发生密码窃取、设备感染，或给出受影响人数。

## 5. 审计把观察窗口向前推了三天

关键证据来自账户审计。9 月 27 日 11:00 的恶意 Worker 上传、路由创建，以及随后 `ssl`、`ipv6` 设置编辑，都关联到同一个账户 API 令牌：`hidden-firefly-98c3`。

它与正常的 Cortex 构建令牌不同。它也是**账户级令牌**，因此一开始在个人 API Tokens 列表里找不到并不奇怪；Cloudflare 对这两类令牌使用不同的管理入口。[2]

扩大查询窗口后，时间线比当天的部署故障更早：

<figure>
  <a href="/assets/incidents/2026-09-27-cloudflare/timeline-zh.svg"><img src="/assets/incidents/2026-09-27-cloudflare/timeline-zh.svg" alt="9月24日创建令牌，25日上传同名 Worker，26日删除十个业务 Worker，27日再次注入并完成阻断、撤销令牌和删除 Worker" width="960" height="1300" loading="lazy" style="width:100%;height:auto;"></a>
  <figcaption>图 2．事件与处置时间线。所有时间均为 UTC+8；节点间距不代表实际经过时长。11:51 的删除时间取自成功 API 回执，其余主要攻击节点取自审计记录。<a href="/assets/incidents/2026-09-27-cloudflare/timeline-zh@2x.png">下载 PNG</a>。</figcaption>
</figure>

9 月 24 日 16:19，令牌创建记录带有控制台上下文。9 月 25 日凌晨，同一令牌已经上传过同名 Worker、配置路由，并修改域名设置；当天晚些时候，又删除过相关路由和 Worker。**这些较早的删除同样关联到该令牌，不能写成站点管理员已经完成过一次清理。** 较早版本的源码没有保全；本文对脚本行为的分析只针对 27 日取得的样本。

较早的审计还记录了一项 Zone 规则集的删除。它的旧内容没有保全，无法据此确定具体改变了什么策略。

9 月 26 日 01:29 至 01:30，同一令牌还删除了十个业务 Worker，涉及此前的 Cortex、文档、代理、站点及 webhook 等服务。删除记录能够确认；各服务具体中断了多久，需要各自的监控和日志，本文没有用推测补齐。

这也限定了对攻击者的描述。令牌 ID 可以关联操作，审计中的 IP 和控制台上下文可以提供线索，但它们没有直接告诉我们背后是谁。最初的账户访问是如何获得的，是否涉及浏览器会话、凭据泄露或某个授权过程，仍没有足够证据定论。本文也没有发现足以归因于 Cloudflare 平台漏洞的证据。

## 6. 处置分成了阻断、撤销和清理

11:29:38，在保存源码和配置证据之后，我们先把恶意 Worker 替换为只做透传的实现：

```js
export default {
  fetch(request) {
    return fetch(request);
  },
};
```

这个临时措施保留原有请求路径，让合法应用继续返回内容，同时移除响应改写和特殊代理逻辑。当时使用的部署凭据能够修改 Worker，却无权管理账户令牌和 Zone 路由；相关 API 返回了 403。先停止执行恶意代码，给后续权限操作留出了时间。

11:45:36，管理员在 **Manage Account → Account API Tokens** 删除目标令牌。随后读取审计，核对到了对应的 `Delete Token` 成功记录及撤销事件。确认的是那个具体令牌，而不是凭名称相似就删除所有构建凭据。

11:51:28，在确认该 Worker 没有业务依赖、资源绑定和自定义域名之后，我们删除了它，API 返回成功。账户 Worker 列表不再包含它，控制台也显示对象不存在。管理员刷新后，报告相关条目已经消失；Zone 全量路由列表仍受现有 API 权限限制，本文不把它写成完成了整个账户的路由审计。

<figure>
  <a href="/assets/incidents/2026-09-27-cloudflare/worker-deleted.png"><img src="/assets/incidents/2026-09-27-cloudflare/worker-deleted.png" alt="Cloudflare 控制台显示 cf-w-d6b620c5 不存在或已被删除" width="1912" height="1312" loading="lazy" style="width:100%;height:auto;"></a>
  <figcaption>图 3．删除后的原始控制台截图，未重绘。截图保留 Worker 名称，不包含 API 密钥。它证明该 Worker 已不存在，不能单独证明整个账户都没有异常配置。</figcaption>
</figure>

## 7. 恢复到了什么程度

处置后的 Mac、Windows User-Agent 检查覆盖中文、英文、日文首页，以及 marketplace、flags 和 robots.txt，共十二项 HTTP 请求，均返回 200，未检出上述注入特征。浏览器检查也没有发现此前的隐藏节点和对应脚本错误。这个验证范围可以支持“所检查的页面与接口已恢复”，不应扩大成对所有访客历史状态的判断。

部署流程也单独验收过：本地 `pnpm cf:deploy` 成功，私有仓库 `dev` 分支的一次真实 GitHub push 触发自动构建并部署成功。安全事件的处置没有替代这项原始任务的验收。

管理员后来将 SSL 设置为 **Full**，并开启 IPv6。审计中有这两项设置编辑的成功记录；后续 DNS 查询已返回 AAAA，绕过本机 HTTP 代理、显式连接一个解析到的 IPv6 地址时，Cortex 返回 HTTP 200，TLS 证书校验通过。

Full 与 Full (Strict) 仍有区别：Full 对 HTTPS 回源加密，但不验证源站证书；Full (Strict) 增加证书校验，需要先确认相关源站满足要求。[3] IPv6 开启则恢复了相应的连接能力。[4] **这两项配置不负责撤销恶意令牌，也不负责删除注入代码。** 旧设置值无法从当时可用的历史接口恢复，所以这里记录的是当前状态，不声称已逐项还原攻击前配置。

截至写作时，Cortex 的两条部署路径、所检查的页面，以及实际 IPv6 访问均正常。更早被删除的其他业务 Worker 没有在本次 Cortex 修复中逐个恢复；账户最初的进入方式仍待追查。这些范围限制需要与“当前服务已恢复”一起保留下来。

## 8. 我会把哪些检查留在以后

这次最值得保留的习惯，是在每次“成功”之后继续问一句：这个结果具体证明了什么。

构建成功证明构建命令跑通；应用源码里没有某段脚本，证明这份源码没有那段脚本；有效证书证明一条经过验证的 TLS 连接。要回答用户最终收到了什么，还得检查公开域名的响应和浏览器 DOM，并把中间的路由、Worker 和账户权限纳入视野。

对我而言，以后的部署验收会保留公开页面检查，而账户侧的变更也应留下可追溯的清单。临时遏制与最终清理需要分开记录：替换成透传代码、撤销写入权限、删除多余资源，各自解决不同的问题。缺少哪一步，都应该能在记录里看出来。

这篇复盘没有一个已经找齐所有答案的结尾。我们找到了改写页面的位置，关联了执行这些操作的令牌，也验证了 Cortex 的恢复。入侵入口和历史访客影响仍是未完成的调查问题。把这些空白写明，是这份记录能够继续被使用的前提。

## 资料与方法

本文依据我与 Codex 协同排查时保存的构建日志、API 回执、审计摘录、Worker 源码、HTML 响应和控制台截图撰写。下载的 Worker 没有在本地重放，条件分支引用的远程载荷也没有被主动获取或执行；实际页面观察与源码能力分析分别记录。文中两张示意图是证据重建，删除截图是原始记录。

可下载[经过字段筛选的时间线 CSV](/assets/incidents/2026-09-27-cloudflare/timeline-extract.csv)。它方便核对本文的事件顺序，但不是未经修改的原始审计文件，也不构成第三方独立取证。公开版本省略账户与 Zone 的完整 ID、令牌 ID、邮箱、操作来源 IP 和原始请求内容；API 密钥、完整恶意源码及外部载荷不随文章发布。

1. Cloudflare, [Custom Domains：与 Routes 的交互](https://developers.cloudflare.com/workers/configuration/routing/custom-domains/#interaction-with-routes)。用于解释请求链路，不用于替代本次事件的现场证据。
2. Cloudflare, [Account-owned API tokens](https://developers.cloudflare.com/fundamentals/api/get-started/account-owned-tokens/)。
3. Cloudflare, [Full](https://developers.cloudflare.com/ssl/origin-configuration/ssl-modes/full/) 与 [Full (Strict)](https://developers.cloudflare.com/ssl/origin-configuration/ssl-modes/full-strict/)。
4. Cloudflare, [IPv6 compatibility](https://developers.cloudflare.com/network/ipv6-compatibility/)。

*初稿记录于 2026 年 9 月 27 日。若后续取得新的入口或影响证据，将以明确标注的更新补充，而不把推断悄悄改写成当时已知的事实。*
