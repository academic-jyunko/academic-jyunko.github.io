---
layout: post
title: "I only meant to fix a deployment: the Cortex Cloudflare incident"
date: 2026-09-27 12:10:00 +0800
author: HsiangNianian
lang: en
permalink: /2026/09/27/cloudflare-worker-incident-en.html
toc: true
excerpt: "I wanted Cortex to deploy from my machine and from GitHub. We fixed the build, then found HTML I had never written. Following it led to another Worker and an audit trail going back three days."
description: "Working through the Cortex Cloudflare incident with Codex: a failed pnpm install, a Worker rewriting HTML, a hard-to-find account token and the checks after cleanup."
image: /assets/incidents/2026-09-27-cloudflare/request-path-en@2x.png
---

[中文版](/2026/09/27/cloudflare-worker-incident-zh.html)

September 27, 2026.

I only meant to fix a deployment. I wanted to be able to publish Cortex from my machine, and have Cloudflare deploy it automatically when I pushed to GitHub. The development branch was already there. Dependency installation was where everything stopped.

We got the build working. Then we found something in the page that I hadn't written. Following it led to another Worker, a token I couldn't find in the dashboard, and ten application Workers that had already been deleted.

There was quite a bit more to fix than I had started with.

I worked through this with Codex. It helped inspect the code, run commands and compare API records; I handled the steps that needed the dashboard. Looking back, I want to keep what we knew at the time separate from what we found later. At the beginning, all we had was an installation log.

## First, the error at nine

The first failed build was at about 09:00:

```text
ERR_PNPM_LOCKFILE_CONFIG_MISMATCH
The current "patchedDependencies" configuration doesn't match
the value found in the lockfile
```

Cortex uses OpenNext to deploy to Cloudflare, with a compatibility patch managed by pnpm. The error pointed to the patch configuration and lockfile. It was reasonable to wonder whether the patch had changed and the lockfile hadn't caught up.

But the same checkout installed locally, with the lockfile frozen there too. Something differed between the two environments. We still had to find it.

We put the diagnostics inside the command Cloudflare actually ran: print the patch file's hash, compare the lockfile entry, then ask pnpm for its version. The log now had two answers:

```text
Environment detection: pnpm@10.11.1
Executed command:      pnpm --version
Command output:        9.10.0
```

Even the pnpm version needed checking beyond the line at the top of the log.

The patch and its lockfile record matched. After pinning pnpm to 10.33.2, the frozen installation passed. We kept `--frozen-lockfile` and fixed the intended pnpm and Node versions in the repository and Cloudflare build variables. The logs still don't explain exactly how the older executable was selected.

Both ways of publishing then worked. Locally, `pnpm cf:deploy` completed. A real push to the private repository's `dev` branch triggered checks, tests, a build and deployment on Cloudflare. We also cleared local environment files that could otherwise end up in OpenNext output, leaving runtime secrets in Cloudflare Secrets.

That took care of the error I had come to fix.

## Except there was something extra in the page

The deployed application opened and ran. Its returned content also contained Betaflight promotional text, links and a hidden top-level heading pointing to `betaflight[.]uk[.]com`.

I hadn't put any of that there.

We searched for the promotional domain, the `content-extra` class and the `__contentLoaded` marker. First in the repository, then in the local build output, then in the Cortex Worker already deployed to Cloudflare. None of them had those indicators. The page returned by the public domain did.

The comparison only made sense after we added another Worker:

| Where we looked | Those injection indicators |
| --- | --- |
| Repository source and local build output | Not found |
| The deployed Cortex Worker's code | Not found |
| HTML returned by the custom domain | Found |
| Another Worker in the account | Matching injection implementation found |

That Worker was called `cf-w-d6b620c5`. It wasn't part of Cortex's deployment configuration. Its route, though, covered a much wider pattern:

```text
*hydroroll.team/*
```

It could receive the request first, then fetch Cortex's response. Cloudflare supports this combination of a route Worker and a Custom Domain Worker: the first can call `fetch(request)` to get the application's response.[1] The extra Worker then used `HTMLRewriter` to append scripts to the HTML.

<figure>
  <a href="/assets/incidents/2026-09-27-cloudflare/request-path-en.svg"><img src="/assets/incidents/2026-09-27-cloudflare/request-path-en.svg" alt="A browser request first reaches the malicious route Worker, which fetches the Cortex response, appends scripts and returns the modified HTML" width="960" height="820" loading="lazy" style="width:100%;height:auto;"></a>
  <figcaption>Figure 1. The request path, reconstructed from saved source and routing configuration. Open the image for the SVG, or <a href="/assets/incidents/2026-09-27-cloudflare/request-path-en@2x.png">download the PNG</a>.</figcaption>
</figure>

That explained how the code could be absent from the repository and present in the page. Redeploying Cortex updated the application. The extra Worker was still in front of it, able to change what it returned.

HTTPS had worked throughout, with certificate validation passing. The rewriting happened inside the request path controlled through this Cloudflare account. In this investigation, the certificate and the page content were telling us about different things.

*One clarification: the name in the promotion doesn't establish any involvement by the Betaflight open-source project. It is retained here as an injection indicator. The suspicious domains are deliberately defanged.*

## Keeping a copy of the code

We saved the Worker's source and configuration before changing it. Those files would be needed to explain what had happened after the page was fixed.

The hidden links were straightforward. The script inserted headings, text and links, then concealed them with off-screen positioning, opacity and related styles. That matched what we had seen on the live page. Further down, there were other branches.

One checked Windows User-Agents, cookies and network-organization information, including whether a visitor appeared to come from a proxy or hosting network, before deciding whether to append an external script. The addresses had a simple XOR transformation applied to them. Decoding revealed a script source under `macrium[.]info`.

Another branch handled paths beginning with `/_r/`. It forwarded requests to an external endpoint, carrying some of the original request context and adding the client's IP, country and city. There was also code to append a PowerShell command. Earlier filters constrained whether that branch could be reached; it needed to be read with the surrounding code.

We inspected the downloaded source and decoded its strings offline. We didn't replay the Worker locally or retrieve and run the external payload it referenced. The hidden content was observed on the live page. The further behaviors described here are capabilities present in the source.

The bypass rules were easy to miss. The Worker passed through `/api`, `/_next`, `robots`, `sitemap` and several kinds of static-resource paths. Some clients received only the hidden promotion. A working API or a page with no visible popup could therefore look quite unremarkable while its HTML had been changed.

For later comparison, this is the fingerprint of the Worker source file we saved:

```text
SHA-256
2659f437dbf42d308d963fd08b9ba7f90ecd8544d403c8deec650bc35f55cc47
```

## The logs went back further

The code explained how the page was being changed. Next came the question of what had permission to put it there.

The account audit linked the Worker upload at 11:00 on September 27, the route creation and the `ssl` and `ipv6` edits immediately afterward to one account API token: `hidden-firefly-98c3`. It was separate from the normal Cortex build token.

We widened the query window. The records went back to September 24.

<figure>
  <a href="/assets/incidents/2026-09-27-cloudflare/timeline-en.svg"><img src="/assets/incidents/2026-09-27-cloudflare/timeline-en.svg" alt="A token is created on September 24; a Worker with the same name is uploaded on September 25; ten application Workers are deleted on September 26; injection and containment follow on September 27" width="960" height="1300" loading="lazy" style="width:100%;height:auto;"></a>
  <figcaption>Figure 2. Events and response, in UTC+8; spacing is not to scale. The 11:51 deletion time comes from the API receipt; the main attack events come from audit records. <a href="/assets/incidents/2026-09-27-cloudflare/timeline-en@2x.png">Download the PNG</a>.</figcaption>
</figure>

At 16:19 on the 24th, the token was created in a dashboard context. Early on the 25th, it had already uploaded a Worker with the same name, configured routes and changed domain settings. Later that day, it deleted an associated route and the Worker.

There is a detail here that I don't want to lose in the retelling. Those earlier deletions were linked to the same token. They weren't a cleanup I had carried out. We also don't have the source from the 25th, so the code analysis above applies to the sample collected on the 27th, not automatically to every earlier version.

An earlier record also showed a zone ruleset being deleted. We don't have its previous contents. For now, that entry tells us only so much.

Between 01:29 and 01:30 on the 26th, the same token deleted ten application Workers. These included the previous Cortex service and services for documentation, proxies, websites and webhooks.

By this point, the morning's installation error could not explain the whole incident. Earlier activity was already in the logs. **We found no evidence that the pnpm failure caused the attack.** It was where this investigation began.

How the token came to be created remains unanswered. The dashboard context and source IPs are leads, but they don't identify a person or establish whether access came through a password, a session or some other authorization. The evidence is also insufficient to attribute the incident to a Cloudflare platform vulnerability.

## “I can't find that token”

There was a permissions problem to work around before cleanup could finish.

The deployment credential we had could modify a Worker. It couldn't manage account tokens or Zone routes; those API requests returned 403. We could at least stop the injection first. At 11:29:38, the malicious Worker's code was replaced with this:

```js
export default {
  fetch(request) {
    return fetch(request);
  },
};
```

The legitimate request path remained in place. Response rewriting and the special proxy logic were gone. The Worker still existed for the moment, running only the pass-through code.

Then I had to find the token in the dashboard.

My reply in the conversation was “I can't find that token.” Later I pasted the list I could see and asked, “Which one?” With the names, timestamps and audit records laid out in an article, this part looks obvious. It was less obvious in front of a list of similarly named credentials.

We needed the **account-owned token**, under **Manage Account → Account API Tokens**, a separate management page from personal API Tokens.[2] The target was `hidden-firefly-98c3`. The normal build token still had a job to do.

At 11:45:36, the target token was deleted. A subsequent audit read showed the matching successful `Delete Token` record and revocation event. After I said it was deleted, we checked the record to make sure it really was the right one.

At 11:51:28, after confirming that the Worker had no application dependencies, resource bindings or custom domains, we deleted it too. The API reported success and the account inventory no longer contained it. I refreshed the dashboard and got the message that the object did not exist.

<figure>
  <a href="/assets/incidents/2026-09-27-cloudflare/worker-deleted.png"><img src="/assets/incidents/2026-09-27-cloudflare/worker-deleted.png" alt="The Cloudflare dashboard reports that the Worker cf-w-d6b620c5 does not exist or has been deleted" width="1912" height="1312" loading="lazy" style="width:100%;height:auto;"></a>
  <figcaption>Figure 3. The original dashboard screenshot after refreshing. This Worker no longer existed; that is what the screenshot establishes.</figcaption>
</figure>

Written up afterward, this became three steps: containment, revocation and removal. Doing it meant changing the code we could change, finding the right dashboard page, then going back to check the deletion results.

## Opening the page again

After cleanup, we repeated the checks. Chinese, English and Japanese homepages, plus marketplace, flags and robots.txt, were tested with Mac and Windows User-Agents: twelve HTTP requests in all. They returned 200 without the known injection indicators. Browser checks no longer found the earlier hidden nodes or corresponding script errors.

We checked the original deployment request too. Local publishing worked, and a real push to `dev` in the private repository triggered a successful automatic build and deployment. That was the task I had started with in the morning.

I later selected **Full** SSL and enabled IPv6. The audit recorded the two settings edits at 12:06 and 12:07, and DNS began returning AAAA records. To keep the local HTTP proxy out of the test, the final check connected explicitly to one resolved IPv6 address. Cortex returned 200 with TLS certificate validation passing.

These settings have their own meanings. Full encrypts HTTPS connections to an origin without validating its certificate; Full (Strict) adds certificate validation and requires a suitable origin certificate.[3] IPv6 provides the corresponding connection capability.[4] The injected code and token had been dealt with in the earlier steps.

Some unfinished work needs to stay in this account of events. Our API credential couldn't enumerate all Zone routes. The dashboard feedback and Worker deletion receipt didn't amount to a full account audit. We couldn't recover the old SSL and IPv6 values through the available history interface. Nor were the ten deleted application Workers individually restored during this Cortex repair. Their downtime still needs their own logs.

At this point, we could confirm that Cortex's deployments, the pages tested and an actual IPv6 connection worked. How access was first obtained, and which script branches earlier visitors encountered, remained unknown.

For future deployments, I want to remember to open the public page as well. Source, build output and the HTML actually returned need to agree. It was the last of those that had something extra this time.

I only meant to fix a deployment. Now “deployment successful” needs a page check after it.

## The records kept

All times are **Asia/Shanghai, UTC+8**, converted from the original `Z` timestamps where necessary. This account draws on the build logs, API receipts, audit extracts, Worker source, HTML responses and dashboard screenshots saved during the investigation. The two diagrams reconstruct those records; the deletion screenshot is unchanged.

A few limits belong with the material. “Not found” in the table refers only to the known injection indicators. A broad route pattern doesn't mean every matching request entered the same malicious branch. Without complete visitor logs or endpoint forensics, we cannot confirm password theft, device infection or an affected-visitor count. Clearing local environment files from build output was an additional deployment correction; no evidence connects those files to the initial access in this incident.

A [redacted timeline CSV](/assets/incidents/2026-09-27-cloudflare/timeline-extract.csv) is included for comparison. It is a derived extract, not a raw audit export or an independent forensic report. Full account, Zone and token IDs, email addresses, source IPs and original request contents are omitted. API secrets, the complete malicious code and external payloads are not published here.

1. Cloudflare, [Custom Domains: interaction with Routes](https://developers.cloudflare.com/workers/configuration/routing/custom-domains/#interaction-with-routes). This documents the routing mechanism; it does not substitute for incident-specific evidence.
2. Cloudflare, [Account-owned API tokens](https://developers.cloudflare.com/fundamentals/api/get-started/account-owned-tokens/).
3. Cloudflare, [Full](https://developers.cloudflare.com/ssl/origin-configuration/ssl-modes/full/) and [Full (Strict)](https://developers.cloudflare.com/ssl/origin-configuration/ssl-modes/full-strict/).
4. Cloudflare, [IPv6 compatibility](https://developers.cloudflare.com/network/ipv6-compatibility/).

*First recorded on September 27, 2026; narration and wording revised the same day, without adding incident findings. Any later evidence about initial access or impact will carry its own update date.*
