---
layout: post
title: "本来只是想修一次部署：Cortex 的 Cloudflare 事件记录"
date: 2026-09-27 12:10:00 +0800
author: HsiangNianian
lang: zh-CN
permalink: /2026/09/27/cloudflare-worker-incident-zh.html
toc: true
excerpt: "本来只想让 Cortex 的本地发布和 GitHub 自动部署都能跑通。后来构建修好了，页面里却多出一段我没写过的东西。顺着它往回查，事情一直到了三天前。"
description: "一次排查记录：从 pnpm 安装失败，到发现前置 Worker 改写 HTML，再到寻找、撤销令牌与恢复 Cortex。"
image: /assets/incidents/2026-09-27-cloudflare/request-path-zh@2x.png
---

[English version](/2026/09/27/cloudflare-worker-incident-en.html)

> 下面尽量把“当时看到的”和“后来才知道的”分开写——不然回头读起来，好像我一早就知道答案似的。其实一早，我手上只有一段装不上依赖的日志。

2026年9月27日 9:00

本来只是想修一次部署。

要求很普通：本地能手动发，代码推到 GitHub 以后，Cloudflare 自己接着部署。Cortex 的开发分支已经推上去了，偏偏卡在装依赖这一步。

后来构建修好了，页面里却多了一段我没写过的东西。再往下查，又查出一个陌生的 Worker、一个怎么也找不到的令牌，还有十个早就被删掉的服务。

越查越多。好吧。

## 先是那个报错

```
ERR_PNPM_LOCKFILE_CONFIG_MISMATCH
The current "patchedDependencies" configuration doesn't match
the value found in the lockfile
```

Cortex 用 OpenNext 部署到 Cloudflare，项目里有一份 pnpm 管的兼容补丁。报错指着补丁和锁文件，第一反应当然是：是不是改了补丁、忘了更新 lockfile？

可同一份代码在本地能装，而且也是冻结安装。

那就把检查塞进 Cloudflare 真正执行的构建命令里——打印补丁哈希、对照锁文件，再让 pnpm 自己报个版本。结果日志里有两种说法：

```
环境探测：pnpm@10.11.1
实际命令：pnpm --version
实际输出：9.10.0
```

……连自己在用哪个 pnpm，都不能信开头那一行。

补丁和锁文件其实是对得上的。固定到 pnpm 10.33.2，冻结安装就过了，`--frozen-lockfile` 也照样留着。之后把仓库和 Cloudflare 构建变量里的 pnpm、Node 版本一起钉死。（至于那个 9.10.0 到底是怎么被选中的，现有日志说不清，我也就没再追。）

两条发布路径都跑通了：本地 `pnpm cf:deploy`；私有仓库 dev 分支来一次真实推送，Cloudflare 检查、测试、构建、发布。顺手清掉了 OpenNext 产物里可能夹带的本地环境文件，运行时密钥老老实实待在 Cloudflare Secrets 里。

到这儿，早上那个报错算是交差了。

## 等等，页面里多了什么

页面能打开，应用也在跑。可浏览器收到的 HTML 里夹着 Betaflight 的推广文字和链接，还有一个藏起来的一级标题，指向 `betaflight[.]uk[.]com`。

我没写过这个。

拿着推广域名、`content-extra` 类名、`__contentLoaded` 标记去搜：仓库，没有；本地构建产物，没有；已经部署的 Cortex Worker，也没有。可公开域名返回的页面里——有。

直到把账户里另一个 Worker 也拉进来对照，才对上：

| 检查的位置 | 有没有这组注入特征 |
|---|---|
| 仓库源码、本地构建产物 | 没有 |
| 已部署的 Cortex Worker | 没有 |
| 自定义域名返回的 HTML | 有 |
| 账户里另一个 Worker | 有，注入就是它做的 |

它叫 `cf-w-d6b620c5`，不在 Cortex 的部署配置里，路由倒是挂得很宽：

```
*hydroroll.team/*
```

也就是说，请求先到它手里，它再去取 Cortex 的响应。Cloudflare 本来就允许 Routes Worker 和 Custom Domain Worker 这样叠着用，前面的调一次 `fetch(request)`，就能拿到后面应用的返回。[1] 它拿到 HTML 以后，用 HTMLRewriter 往里追加脚本。

<figure>
  <a href="/assets/incidents/2026-09-27-cloudflare/request-path-zh.svg"><img src="/assets/incidents/2026-09-27-cloudflare/request-path-zh.svg" alt="浏览器先访问恶意路由 Worker，后者取得 Cortex 的 HTML 后追加脚本，再把修改后的页面返回浏览器" width="960" height="820" loading="lazy" style="width:100%;height:auto;"></a>
  <figcaption>图 1．按保存的源码和路由配置重建的请求链路。点击可查看 SVG，或<a href="/assets/incidents/2026-09-27-cloudflare/request-path-zh@2x.png">下载 PNG</a>。</figcaption>
</figure>

这下“仓库里没有、页面里有”就说得通了：我重新部署 Cortex，更新的只是 Cortex 自己，挡在前面的那个照样改它的响应。

HTTPS 一直是好的，证书校验一次也没报错——改写本来就发生在这个账户自己的请求链路里，证书管不着页面里写了什么。

（多说一句：推广里出现 Betaflight 这个名字，不代表和那个开源飞控项目有什么关系。文中的域名都已断链。）

## 先把它留下来

找到 Worker，第一件事不是删，是存。源码和配置先保存下来，以后要靠这些东西解释发生了什么，光顾着把页面修好是不够的。

藏链接的手法很朴素：往页面里塞标题、文字和链接，再用离屏定位、透明度之类的样式藏起来。和现场看到的对得上。

但它不止做了这些。

- 有一段会看 Windows User-Agent、Cookie、网络归属，判断访客像不像来自代理或机房，再决定要不要追加一个外部脚本。地址做了简单的 XOR 混淆，解出来是 `macrium[.]info`。
- 另一段处理 `/_r/` 路径，把请求转发到外部端点，捎上原请求的部分信息，再附上客户端 IP、国家和城市。
- 源码里还有追加 PowerShell 命令的逻辑。不过它前面还有好几层过滤，到底能不能走到那儿，得连起来读。

而且它很会躲：`/api`、`/_next`、robots、sitemap 和各种静态资源一律放行，有些客户端只会收到推广内容。接口照样 200，页面也不弹窗——谁会想到 HTML 里多了东西呢。

这些代码我们只做了静态阅读和字符串解码，没在本地跑那个 Worker，也没去下载、运行它引用的外部载荷。页面上的隐藏内容是亲眼看到的；后面那几条，只能说源码“有这个能力”，不能说“已经这样做了”。

样本指纹，留着以后对照：

```
SHA-256
2659f437dbf42d308d963fd08b9ba7f90ecd8544d403c8deec650bc35f55cc47
```

## 往前翻日志

代码说明了页面是怎么被改的。下一个问题是：谁有权限把它放进来？

账户审计里，9 月 27 日 11:00 的 Worker 上传、路由创建，以及紧跟着的 ssl、ipv6 设置修改，全都挂在同一个账户 API 令牌名下——`hidden-firefly-98c3`。不是 Cortex 正常构建用的那个。

把查询时间往前推，一直推到了 9 月 24 日。

<figure>
  <a href="/assets/incidents/2026-09-27-cloudflare/timeline-zh.svg"><img src="/assets/incidents/2026-09-27-cloudflare/timeline-zh.svg" alt="9月24日创建令牌，25日上传同名 Worker，26日删除十个业务 Worker，27日再次注入并完成阻断、撤销令牌和删除 Worker" width="960" height="1300" loading="lazy" style="width:100%;height:auto;"></a>
  <figcaption>图 2．事件与处置时间线，UTC+8，节点间距不代表经过时长。11:51 的删除时间取自 API 回执，主要攻击节点取自审计记录。<a href="/assets/incidents/2026-09-27-cloudflare/timeline-zh@2x.png">下载 PNG</a>。</figcaption>
</figure>

- 24 日 16:19，这个令牌被创建，记录带着控制台上下文；
- 25 日凌晨，它已经上传过同名 Worker、配过路由、改过域名设置；同一天晚些时候，又把相关路由和 Worker 删掉了；
- 26 日 01:29—01:30，还是它，删掉了十个业务 Worker——之前的 Cortex，还有文档、代理、站点、webhook 等等，都在里面。

看到这里我愣了一下。25 日那次“删除”也是这个令牌做的，不是我清理过一轮。那天的代码没留下来，所以现在分析的只是 27 日拿到的样本，不能直接当成 25 日跑的也是同一份。旧记录里还有一条 Zone 规则集删除，规则原来写了什么，同样没留下。

早上那个 pnpm 报错，到这时候已经解释不了整件事了。它只是我开始查的地方，没有证据说明它和攻击有关系。

令牌最初是怎么来的？审计没给答案。控制台上下文和来源 IP 是线索，但凭这些定不了是谁，也分不清漏掉的是密码、会话还是别的授权。现有证据也不足以怪到 Cloudflare 平台漏洞头上。

## “找不到该令牌”

真动手的时候，先撞上的是权限。

我手里那份部署凭据能改 Worker，却管不了账户令牌和 Zone 路由，一调就是 403。那就先做能做的：11:29:38，把恶意 Worker 的代码换成这几行——

```js
export default {
  fetch(request) {
    return fetch(request);
  },
};
```

请求路径不变，应用照常返回，改写和代理的逻辑全部拿掉。它暂时还挂在那儿，只是变成了透传。

然后轮到我去控制台删令牌。

对着控制台翻了一圈，我的第一反应是：找不到该令牌。过了一会儿，盯着眼前那一列令牌，又冒出一句：哪个？

现在把名字、时间、审计记录排成一篇文章，看着一清二楚；当时对着一列名字都差不多的凭据，可没这么清楚。

要找的是账户级令牌，入口在 **Manage Account → Account API Tokens**，和个人的 API Tokens 是分开的。[2] 目标是 `hidden-firefly-98c3`，正常构建用的那个得留着。

- 11:45:36，目标令牌删除。审计里随后有对应的 Delete Token 成功记录和撤销事件——我说“删了”之后，还是回头对了一遍，确认删的确实是它。
- 11:51:28，确认那个 Worker 没有业务依赖、资源绑定和自定义域名后，把它也删了。API 返回成功，列表里没有了；刷新控制台，提示对象不存在。

<figure>
  <a href="/assets/incidents/2026-09-27-cloudflare/worker-deleted.png"><img src="/assets/incidents/2026-09-27-cloudflare/worker-deleted.png" alt="Cloudflare 控制台显示 cf-w-d6b620c5 不存在或已被删除" width="1912" height="1312" loading="lazy" style="width:100%;height:auto;"></a>
  <figcaption>图 3．刷新后看到的控制台，原始截图。这个 Worker 已不存在；截图本身只确认这一点。</figcaption>
</figure>

后来这一段被整理成“阻断、撤销、清理”三步，听起来很利落。实际上就是：先改掉能改的代码，再在控制台里找半天入口，最后回过头核对删没删对。

## 再打开一次页面

清理完，重新检查一遍：中文、英文、日文首页，marketplace、flags、robots.txt，Mac 和 Windows 两种 User-Agent 交叉，一共十二个请求，全部 200，没再出现那组注入特征。浏览器里，之前的隐藏节点和对应的脚本报错也没了。

最初的需求也没落下：本地手动发布成功；dev 分支的真实推送触发自动构建和发布，也成功。到这时候，早上那个要求才算真正做完。

之后我把 SSL 设成 Full，开了 IPv6。审计里有 12:06、12:07 两次设置修改，DNS 也开始返回 AAAA。为了不让本机代理干扰结果，最后绕开代理，直接连一个解析出来的 IPv6 地址——Cortex 返回 200，TLS 校验通过。

（顺带记一下这两个设置管什么：Full 会加密回源，但不校验源站证书，Full (Strict) 才校验，前提是源站证书合格；[3] IPv6 管的就是 IPv6 连接。[4] 它们和注入、令牌都没关系，那两件事前面已经处理掉了。）

## 还没做完的

有几件事得老实写下来：

- 没有权限用 API 把整个 Zone 的路由列一遍。控制台刷新和删除回执，代替不了一次完整的账户审计；
- 攻击前 SSL、IPv6 原来是什么值，当时能用的历史接口里找不回来；
- 那十个被删掉的业务 Worker，这次没有逐个恢复，它们各自停了多久，得去翻各自的日志。

所以能确认的只有这些：Cortex 现在的部署、检查过的那些页面、实际的 IPv6 访问，都正常了。对方最初是怎么进来的，过去哪些访客碰到了哪条脚本分支——还不知道。

以后再发版，我会记得把公开页面打开看一眼。源码、构建产物、实际返回的 HTML，三处要对得上。这次就是最后那一处，多了东西。

本来只是想修一次部署。现在每次“部署成功”后面，都得再补一次页面检查了。

## 附：材料与边界

时间统一为北京时间（Asia/Shanghai，UTC+8），原始日志里的 Z 时间已经换算。经过依据排查时保存的构建日志、API 回执、审计摘录、Worker 源码、HTML 响应和控制台截图整理；图 1、图 2 按这些材料重建，图 3 是原始截图。

- 表里的“没有”，只针对那组已知的注入特征；
- 路由挂得宽，不代表所有匹配的请求都走了同一条恶意分支；
- 没有完整的访客日志和终端取证，没法确认有没有发生密码窃取、设备感染，也给不出受影响人数；
- 构建时清理本地环境文件，是顺手修的部署配置，目前没有证据说明它和这次入侵有关。

另附[脱敏后的时间线 CSV](/assets/incidents/2026-09-27-cloudflare/timeline-extract.csv)，方便核对；它是摘录，不是原始审计文件，也不是第三方取证报告。账户、Zone、令牌的完整 ID，邮箱、来源 IP、原始请求内容，以及 API 密钥、完整恶意代码和外部载荷，都没有公开。

1. Cloudflare, Custom Domains：与 Routes 的交互。只用来解释请求链路，不能代替这次的现场证据。
2. Cloudflare, Account-owned API tokens.
3. Cloudflare, Full 与 Full (Strict).
4. Cloudflare, IPv6 compatibility.

> 初稿写于 2026 年 9 月 27 日，同日改过叙述和措辞，事实没有增加。以后要是查到新的入口或影响证据，会另外标出更新时间。

---

> **更新（2026-10-28）**：后来发现，问题不只在 hydroroll.team。jyunko.cn 这个 Zone 下的 diary 和 ddl 两个子站，也被塞进了同一套 Betaflight 推广内容——一个站放在 GitHub Pages，一个放在 Vercel，源站都是干净的，手法和 Cortex 这次一模一样。查下来是这个 Zone 里还挂着一个异常的 Worker 和对应的 API 令牌，当时那一轮清理没覆盖到这里。现在两个都已经删除，复查页面也恢复正常了。所以那句“没有权限枚举整个 Zone 的路由”，这次算是真的付出了代价。
