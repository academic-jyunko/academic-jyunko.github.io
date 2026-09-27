---
layout: post
title: "本来只是想修一次部署：Cortex 的 Cloudflare 事件记录"
date: 2026-09-27 12:10:00 +0800
author: HsiangNianian
lang: zh-CN
permalink: /2026/09/27/cloudflare-worker-incident-zh.html
toc: true
excerpt: "本来只想让 Cortex 的本地发布和 GitHub 自动部署都能跑通。后来构建修好了，页面里却多出一段我没写过的东西。顺着它往回查，事情一直到了三天前。"
description: "一次和 Codex 共同排查的记录：从 pnpm 安装失败，到发现前置 Worker 改写 HTML，再到寻找、撤销令牌与恢复 Cortex。"
image: /assets/incidents/2026-09-27-cloudflare/request-path-zh@2x.png
---

[English version](/2026/09/27/cloudflare-worker-incident-en.html)

2026 年 9 月 27 日。

本来只是想修一次部署。我的要求也很普通：本地可以手动发，代码推到 GitHub 后，Cloudflare 能自己接着部署。Cortex 的开发分支已经推上去了，偏偏安装依赖这一步过不去。

后来，构建修好了。页面里却多出了一段我没写过的东西。再往下查，查到了另一个 Worker、一个找不到的令牌，还有此前被删掉的十个服务。

事情就是这样越查越多的。

这次是我和 Codex 一起排查的。命令、源码和 API 记录由它协助检查，需要进入控制台的操作由我来做。现在回头整理，得把当时看到的和后来查到的分开，不然很容易写得像是一开始就知道答案。其实一开始，我们只有一段安装失败的日志。

## 九点，先看那个报错

最早的构建在 09:00 左右失败，报的是这个：

```text
ERR_PNPM_LOCKFILE_CONFIG_MISMATCH
The current "patchedDependencies" configuration doesn't match
the value found in the lockfile
```

Cortex 用 OpenNext 部署到 Cloudflare，项目里有一份由 pnpm 管理的兼容性补丁。报错指向补丁配置和锁文件，顺着它想下去，自然会怀疑是不是改过补丁，却忘了更新锁文件。

但同一份代码在本地能装上，而且也是冻结安装。哪里不一样，总得找出来。

于是把检查放进 Cloudflare 真正执行的构建命令里：打印补丁文件的哈希，对照锁文件，再让它自己报一次版本。日志里出现了两种说法：

```text
环境探测：pnpm@10.11.1
实际命令：pnpm --version
实际输出：9.10.0
```

原来连正在用哪个 pnpm，都不能只看开头那一行。

补丁和锁文件里的记录能对上。固定使用 pnpm 10.33.2 后，冻结安装就通过了；`--frozen-lockfile` 也保留了下来。仓库和 Cloudflare 构建变量里的 pnpm、Node 版本随后一并固定。至于那个旧版本究竟是怎么被选中的，现有日志还没解释清楚。

两条发布路径也分别跑通了。本地用 `pnpm cf:deploy`；私有仓库的 `dev` 分支收到真实推送后，Cloudflare 依次检查、测试、构建，再发布。顺手还清理了 OpenNext 产物可能夹带的本地环境文件，让运行时密钥留在 Cloudflare Secrets 里。

写到这里，原先找我麻烦的那个报错算是解决了。

## 可是，页面里多了一段东西

部署后的页面能打开，应用也在运行。但浏览器收到的内容里夹着 Betaflight 推广文本和链接，还有一个隐藏的一级标题，指向 `betaflight[.]uk[.]com`。

这些东西从哪来的。

我们拿着推广域名、`content-extra` 类名和 `__contentLoaded` 标记去找。先搜仓库，再搜本地构建产物，接着看已经部署到 Cloudflare 的 Cortex Worker。都没找到。公开域名返回的页面里，却有。

后来把另一份 Worker 也加进来，结果才对上：

| 检查的位置 | 是否有这组注入特征 |
| --- | --- |
| 仓库源码、本地构建产物 | 没找到 |
| 已部署的 Cortex Worker 代码 | 没找到 |
| 自定义域名返回的 HTML | 有 |
| 账户里的另一个 Worker | 有对应的注入实现 |

这个 Worker 叫 `cf-w-d6b620c5`，不在 Cortex 的部署配置里，路由倒是挂得很宽：

```text
*hydroroll.team/*
```

它可以先接到请求，再去取 Cortex 的响应。Cloudflare 支持 Routes Worker 和 Custom Domain Worker 这样组合，前面的 Worker 调用 `fetch(request)`，就能拿到后面应用返回的内容。[1] 这份代码拿到 HTML 后，又用 `HTMLRewriter` 往里面追加了脚本。

<figure>
  <a href="/assets/incidents/2026-09-27-cloudflare/request-path-zh.svg"><img src="/assets/incidents/2026-09-27-cloudflare/request-path-zh.svg" alt="浏览器先访问恶意路由 Worker，后者取得 Cortex 的 HTML 后追加脚本，再把修改后的页面返回浏览器" width="960" height="820" loading="lazy" style="width:100%;height:auto;"></a>
  <figcaption>图 1．按保存的源码和路由配置重建的请求链路。点击可查看 SVG，或<a href="/assets/incidents/2026-09-27-cloudflare/request-path-zh@2x.png">下载 PNG</a>。</figcaption>
</figure>

到这里，仓库里找不到、页面里却有的矛盾就解开了。重新部署 Cortex，只更新了应用本身。那个额外的 Worker 还在前面，照样可以改它的响应。

HTTPS 也一直正常，证书校验没有报错。毕竟改写就发生在这个 Cloudflare 账户控制的请求链路里。这次检查里，证书和页面内容各自说的是不同的事。

*补一句：推广里出现 Betaflight 这个名字，并不能把事情算到那个开源项目头上。这里保留的是注入特征，域名也都作了断链处理。*

## 把那份代码留下来

找到 Worker 后，先保存源码和配置，再改它。后面要靠这些文件解释发生过什么，不能只顾着把页面修好。

隐藏链接的实现很直接：往页面里塞标题、文本和链接，再用离屏定位、透明度等样式藏起来。它和现场看到的内容能对上。但继续往下读，还有别的分支。

有一段会检查 Windows User-Agent、Cookie、网络组织信息，以及访问者是否像是来自代理或托管网络，再决定要不要追加外部脚本。地址还做了简单的 XOR 混淆，解码后能看到 `macrium[.]info`。

另一段处理 `/_r/` 路径，把请求转发到外部端点，带上原请求的部分信息，再附加客户端 IP、国家和城市。源码里还放着追加 PowerShell 命令的逻辑，不过它前面有其他过滤条件，能不能走到那里，得连着读。

这些代码我们只做了静态阅读和字符串解码，没有在本地重放下载的 Worker，也没有去获取、运行它引用的外部载荷。页面上的隐藏内容是实际观察到的；那些更进一步的行为，记录的是源码具备的能力。

还有个容易漏掉的地方：它会放过 `/api`、`/_next`、`robots`、`sitemap` 和多种静态资源路径。某些客户端也只会收到隐藏推广内容。这样一来，接口返回 200、页面没有弹窗，都很难让人察觉还有东西夹在 HTML 里。

保存下来的 Worker 文件指纹放在这里，方便以后对照：

```text
SHA-256
2659f437dbf42d308d963fd08b9ba7f90ecd8544d403c8deec650bc35f55cc47
```

## 日志还得往前翻

代码解释了页面是怎么被改的。接下来要找的是，谁有权限把它放进去。

账户审计里，9 月 27 日 11:00 的 Worker 上传、路由创建，以及紧接着的 `ssl`、`ipv6` 设置编辑，都关联到同一个账户 API 令牌：`hidden-firefly-98c3`。它和正常的 Cortex 构建令牌不是同一个。

再把查询时间往前推，记录一直到了 9 月 24 日。

<figure>
  <a href="/assets/incidents/2026-09-27-cloudflare/timeline-zh.svg"><img src="/assets/incidents/2026-09-27-cloudflare/timeline-zh.svg" alt="9月24日创建令牌，25日上传同名 Worker，26日删除十个业务 Worker，27日再次注入并完成阻断、撤销令牌和删除 Worker" width="960" height="1300" loading="lazy" style="width:100%;height:auto;"></a>
  <figcaption>图 2．事件与处置时间线，UTC+8，节点间距不代表经过时长。11:51 的删除时间取自 API 回执，主要攻击节点取自审计记录。<a href="/assets/incidents/2026-09-27-cloudflare/timeline-zh@2x.png">下载 PNG</a>。</figcaption>
</figure>

24 日 16:19，这个令牌被创建，记录带有控制台上下文。25 日凌晨，它已经上传过同名 Worker、配置过路由、改过域名设置；当天晚些时候，又删除了相关路由和 Worker。

这里得停一下。那些较早的删除也关联着同一个令牌，不能把它们当成我已经做过一次清理。25 日那份代码没有保存下来，所以现在分析的是 27 日拿到的样本，不能直接认定早先跑的也是同一份代码。

旧记录里还有一项 Zone 规则集删除。规则集原先写了什么，没有留下来，这一条暂时只能记到这里。

26 日 01:29 至 01:30，同一个令牌又删掉了十个业务 Worker。此前的 Cortex，以及文档、代理、站点、webhook 等服务，都在其中。

看到这里，当天早上的安装报错已经解释不了整件事了。更早的操作就留在日志里。**没有证据表明那次 pnpm 故障导致了攻击**；部署失败是这次排查的起点。

至于令牌最初怎么来的，审计还没给出答案。控制台上下文和来源 IP 是线索，但无法单凭它们确定是谁，或者确定到底泄露了密码、会话，还是别的授权。现有证据也不足以把原因归到 Cloudflare 的平台漏洞上。

## “找不到该令牌”

真正动手处理时，还有权限这一关。

当时用的部署凭据能修改 Worker，却不能管理账户令牌和 Zone 路由，相关 API 请求都是 403。先能做的是把注入停掉。11:29:38，恶意 Worker 的代码被替换成了这一小段：

```js
export default {
  fetch(request) {
    return fetch(request);
  },
};
```

保留原有请求路径，让应用继续返回内容，把改写响应和特殊代理的逻辑拿掉。那个 Worker 暂时还在，但已经换成了透传代码。

然后轮到我去控制台删令牌。

我在对话里回的是：“找不到该令牌”。后来又把眼前的令牌列表贴出来，问了句“哪个”。现在把名字、时间和审计记录排在一篇文章里，看起来很清楚；当时面对一列名称相近的凭据，就没这么顺了。

要找的是**账户级令牌**，入口在 **Manage Account → Account API Tokens**，和个人 API Tokens 分开。[2] 对准的是 `hidden-firefly-98c3`，正常构建用的令牌还要留着。

11:45:36，目标令牌被删除。随后读到的审计记录里，有对应的 `Delete Token` 成功记录，也有撤销事件。我说删了之后，还得再对一次，确认删的确实是它。

11:51:28，确认那个 Worker 没有业务依赖、资源绑定和自定义域名后，它也被删除了。API 返回成功，账户列表里不再有它。我刷新控制台，看到的是对象不存在的提示。

<figure>
  <a href="/assets/incidents/2026-09-27-cloudflare/worker-deleted.png"><img src="/assets/incidents/2026-09-27-cloudflare/worker-deleted.png" alt="Cloudflare 控制台显示 cf-w-d6b620c5 不存在或已被删除" width="1912" height="1312" loading="lazy" style="width:100%;height:auto;"></a>
  <figcaption>图 3．刷新后看到的控制台，原始截图。这个 Worker 已不存在；截图本身只确认这一点。</figcaption>
</figure>

这一段后来被整理成“阻断、撤销、清理”三个步骤。实际做起来，是先改掉能改的代码，再找到控制台里那个入口，最后回头核对删除结果。

## 再打开一次页面

清理之后，检查重新做了一遍。中文、英文、日文首页，以及 marketplace、flags 和 robots.txt，用 Mac、Windows User-Agent 组合测试，共十二项请求，都返回 200，没再检出那组注入特征。浏览器里也没再看到先前的隐藏节点和对应脚本错误。

最初要修的部署也没有落下：本地手动发布成功，私有仓库 `dev` 分支的真实推送触发自动构建和发布，同样成功。到这里，才算把早上提出的要求做完。

我后来把 SSL 设为 **Full**，开启了 IPv6。审计里有 12:06、12:07 的两次设置编辑，DNS 也开始返回 AAAA。为免本机代理影响结果，最后还绕过代理，显式连接一个解析到的 IPv6 地址；Cortex 返回 200，TLS 校验通过。

这些设置也有各自的含义。Full 会加密 HTTPS 回源，但不验证源站证书；Full (Strict) 才多出证书校验，前提是源站证书满足要求。[3] IPv6 管的是相应的连接能力。[4] 注入代码和令牌，则是在前面的步骤里处理掉的。

还有几处没做完的事，得留在记录里。我们没有权限用 API 枚举整个 Zone 的路由，控制台刷新后的反馈与 Worker 删除回执，也代替不了一次完整的账户审计。攻击前 SSL、IPv6 的旧值没能从当时可用的历史接口里找回来。那十个被删掉的业务 Worker，也没有在这次 Cortex 修复里逐个恢复，各自停了多久仍要查各自的日志。

所以，写到这里能确认的是 Cortex 当前的部署、检查过的页面和实际 IPv6 访问已经正常。最初怎么进去的，过去哪些访客碰到了哪一条脚本分支，还不知道。

以后再发版，我会记得把公开页面也打开看看。源码、构建产物、实际返回的 HTML，三处得对得上。这次就是最后那一处，多出了东西。

本来只是想修一次部署。现在连那句“部署成功”，后面也得补上一次页面检查了。

## 留下来的材料

时间统一为 **Asia/Shanghai，UTC+8**，原始日志里的 `Z` 时间已换算。上面的经过依据排查时保存的构建日志、API 回执、审计摘录、Worker 源码、HTML 响应和控制台截图整理。两张示意图按这些材料重建，删除截图保留原样。

几处判断的范围也一并放在这里：表格中的“没找到”只针对那组已知注入特征；路由写得宽，不等于所有匹配请求都走了同一条恶意分支。我们没有完整的访客日志和终端取证，无法确认是否发生密码窃取、设备感染，也给不出受影响人数。构建时清理本地环境文件，是额外修正部署配置，尚无证据将这些文件与本次入侵入口联系起来。

另附[脱敏后的时间线 CSV](/assets/incidents/2026-09-27-cloudflare/timeline-extract.csv)。它是方便核对的摘录，不是原始审计文件或第三方取证报告。公开材料省略了账户、Zone 和令牌的完整 ID、邮箱、来源 IP 及原始请求内容；API 密钥、完整恶意代码和外部载荷也没有放进来。

1. Cloudflare, [Custom Domains：与 Routes 的交互](https://developers.cloudflare.com/workers/configuration/routing/custom-domains/#interaction-with-routes)。用于解释请求链路，不用于替代本次事件的现场证据。
2. Cloudflare, [Account-owned API tokens](https://developers.cloudflare.com/fundamentals/api/get-started/account-owned-tokens/)。
3. Cloudflare, [Full](https://developers.cloudflare.com/ssl/origin-configuration/ssl-modes/full/) 与 [Full (Strict)](https://developers.cloudflare.com/ssl/origin-configuration/ssl-modes/full-strict/)。
4. Cloudflare, [IPv6 compatibility](https://developers.cloudflare.com/network/ipv6-compatibility/)。

*初稿记录于 2026 年 9 月 27 日，同日修订叙述与措辞。事件事实未作新增。往后若查到新的入口或影响证据，会另标更新时间。*
