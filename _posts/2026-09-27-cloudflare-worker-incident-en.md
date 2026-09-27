---
layout: post
title: "From a failed deployment to injected HTML: the Cortex Cloudflare incident"
date: 2026-09-27 12:10:00 +0800
author: HsiangNianian
lang: en
permalink: /2026/09/27/cloudflare-worker-incident-en.html
toc: true
excerpt: "A pnpm error led me back to the build system. HTML that did not belong to the application led us further, into Cloudflare account configuration. This report reconstructs the discovery, evidence and containment of an edge-script injection."
description: "An evidence-based account of the Cortex Cloudflare Workers injection: deployment failure, a zone-wide route, an account token, containment and the limits of recovery verification."
image: /assets/incidents/2026-09-27-cloudflare/request-path-en@2x.png
---

[中文版](/2026/09/27/cloudflare-worker-incident-zh.html)

On September 27, 2026, I was trying to fix a deployment. Cortex's private development branch had been pushed to GitHub, but Cloudflare stopped while installing dependencies. The error looked familiar: the lockfile and the configured package patches did not agree.

Once the build was fixed, we checked the page it served. The application opened, but its HTML contained unrelated Betaflight promotional material, including a hidden top-level heading. The text was absent from the repository and the local build output. Following those extra bytes back through the delivery path led us to another Worker in the Cloudflare account and a route covering the domain.

This is a case report based on the evidence retained during that investigation. It separates what we observed, what the recovered code could do, and what remains unknown. All times below are **Asia/Shanghai, UTC+8**; timestamps ending in `Z` in the original records have been converted. The deployment failure and the injection were both real, but **we found no evidence that the pnpm failure caused the attack**.

## 1. The build: a detected version was not the executed version

The initial failure occurred at about 09:00:

```text
ERR_PNPM_LOCKFILE_CONFIG_MISMATCH
The current "patchedDependencies" configuration doesn't match
the value found in the lockfile
```

Cortex uses OpenNext for deployment to Cloudflare and maintains a compatibility patch through pnpm. A plausible first explanation was that someone had changed the patch without refreshing the lockfile. Yet the same checkout passed a frozen installation locally. That discrepancy needed an experiment, not just a regenerated lockfile.

We added diagnostics to the command running inside Cloudflare: the patch file's hash, its lockfile entry, and the output of `pnpm --version`. The resulting log showed:

```text
Environment detection: pnpm@10.11.1
Executed command:      pnpm --version
Command output:        9.10.0
```

In this build environment, the detected tool version did not identify the executable ultimately invoked by the command. The patch content and its recorded hash showed no unexpected change. Pinning pnpm to 10.33.2 allowed the same frozen installation to complete. We kept `--frozen-lockfile` and made the intended pnpm and Node versions explicit in both the repository and Cloudflare's build variables. This established a version-selection and patch-lock compatibility problem; it did not establish exactly why the older executable had been selected.

Local deployment and a real GitHub-push deployment then succeeded. The automatic path ran checks, tests, the build and publication in sequence. We also removed local environment files that could otherwise be copied into OpenNext output, leaving runtime secrets in Cloudflare Secrets. That tightened a deployment boundary. It is not evidence that the attacker obtained credentials from those files.

Had acceptance stopped at the successful build, the investigation would have stopped too.

## 2. The page worked, and the extra content was real

The deployed application rendered, but its returned content included hidden promotional text and links to `betaflight[.]uk[.]com`. The name appeared in the injected material. Its presence does not establish any involvement by the Betaflight open-source project.

We compared a small, consistent set of indicators across the delivery path: the promotional domain, the `content-extra` class and the `__contentLoaded` marker.

| Object inspected | Those injection indicators |
| --- | --- |
| Repository source and local build output | Not found |
| The deployed Cortex Worker's code | Not found |
| HTML returned by the custom domain | Found |
| Another Worker's source in the same account | Corresponding injection implementation found |

“Not found” refers to those indicators, not a proof that every part of the application was secure. The comparison was nevertheless enough to redirect the investigation: something else could execute between the application response and the browser.

HTTPS provided no obvious warning. Certificate validation succeeded. That became understandable once the request path was known: the response was being modified inside infrastructure controlled through the legitimate Cloudflare account. The browser's TLS connection did not need to fail for the HTML to change.

## 3. Another Worker stood in front of the application

The account contained a Worker named `cf-w-d6b620c5`. It was absent from Cortex's deployment configuration, but it had a route with this pattern:

```text
*hydroroll.team/*
```

That expanded the possible scope beyond one application to matching requests across the domain. Actual impact still depended on paths, route precedence and code branches. A broad route pattern is not a count of affected visitors.

Cloudflare supports running a route Worker in front of a Worker attached to a Custom Domain. The first Worker can call `fetch(request)` to obtain the second Worker's response.[1] Here, the additional Worker used that position to retrieve application HTML and append scripts with `HTMLRewriter`.

<figure>
  <a href="/assets/incidents/2026-09-27-cloudflare/request-path-en.svg"><img src="/assets/incidents/2026-09-27-cloudflare/request-path-en.svg" alt="A browser request first reaches the malicious route Worker, which fetches the Cortex response, appends scripts and returns the modified HTML" width="960" height="820" loading="lazy" style="width:100%;height:auto;"></a>
  <figcaption>Figure 1. The request path reconstructed from retained source and routing configuration. It explains why redeploying Cortex alone would leave the injection in place. Select the image to open the SVG, or <a href="/assets/incidents/2026-09-27-cloudflare/request-path-en@2x.png">download the PNG</a>.</figcaption>
</figure>

The repository and application Worker could therefore lack the observed injection while the public response still contained it. An application deployment does not automatically remove a separate Worker attached through account configuration.

## 4. The recovered code contained more than hidden links

After preserving the Worker, we inspected its source and decoded strings offline. We did not replay the downloaded Worker locally or fetch and run the external payload referenced by its conditional delivery branch. The hidden content described above was observed on the live page. Several distinct behaviors were present:

- **Hidden content injection.** The script inserted promotional links, headings and text, hiding them with off-screen positioning, opacity and related styles. This implementation matched the content observed in the public response.
- **Conditional script delivery.** A separate branch used Windows User-Agent checks, cookies, network-organization information and proxy or hosting-network checks to decide whether to retrieve and append an external script. The external addresses were obscured with a simple XOR string transformation.
- **A special-path proxy.** Requests beginning with `/_r/` could be forwarded to an external endpoint. The implementation forwarded request context and added client IP, country and city information.
- **A command-bearing response branch.** The source also contained logic to append a PowerShell command for a particular User-Agent condition. Earlier filters constrained its reachability. The presence of this code does not establish that anyone executed the command during the incident.

One decoded script source was under `macrium[.]info`. Domains in this report are deliberately defanged; they are indicators, not recommended destinations. The retained Worker source file has this fingerprint:

```text
SHA-256 of the retained Worker source file
2659f437dbf42d308d963fd08b9ba7f90ecd8544d403c8deec650bc35f55cc47
```

The bypass rules mattered to verification. The code passed through `/api`, `/_next`, `robots`, `sitemap` and numerous static-resource paths. Some clients received the hidden promotional content without entering the additional payload branch. A healthy API response, or the absence of a visible popup on one machine, was therefore a weak test for the integrity of HTML pages.

These findings describe **capabilities visible in the recovered source**. We do not have complete visitor logs or endpoint forensics. The evidence does not justify a claim of confirmed password theft, infected devices or a measured victim count.

## 5. Audit records moved the start of the story back three days

Account audit records supplied the strongest link between resources and actions. The malicious Worker upload at 11:00 on September 27, the route creation and the subsequent `ssl` and `ipv6` edits were associated with the same account API token: `hidden-firefly-98c3`.

This was different from the normal Cortex build token. It was also an **account-owned token**, which explained why it did not appear in the personal API Tokens list initially consulted. Cloudflare provides separate management surfaces for the two token types.[2]

Expanding the query window revealed activity that predated the deployment failure:

<figure>
  <a href="/assets/incidents/2026-09-27-cloudflare/timeline-en.svg"><img src="/assets/incidents/2026-09-27-cloudflare/timeline-en.svg" alt="A token is created on September 24; a Worker with the same name is uploaded on September 25; ten application Workers are deleted on September 26; injection and containment follow on September 27" width="960" height="1300" loading="lazy" style="width:100%;height:auto;"></a>
  <figcaption>Figure 2. Incident and response chronology. All times are UTC+8; spacing is not proportional to elapsed time. The 11:51 deletion time comes from a successful API receipt. The main attack events come from audit records. <a href="/assets/incidents/2026-09-27-cloudflare/timeline-en@2x.png">Download the PNG</a>.</figcaption>
</figure>

At 16:19 on September 24, the token-creation record carried a dashboard context. Early on September 25, the same token had already uploaded a Worker under the same name, configured routing and changed zone settings. Later that day, it deleted an associated route and the Worker. **Those earlier deletions were associated with the same token; they should not be retold as a successful cleanup by the site administrator.** The earlier source versions were not preserved; the behavioral analysis in this report applies to the sample obtained on September 27.

The earlier audit also records a zone-ruleset deletion. Its previous contents were not preserved, so the specific policy change cannot be established from this record alone.

Between 01:29 and 01:30 on September 26, the token also deleted ten application Workers, including the previous Cortex service and services used for documentation, proxying, websites and webhooks. The deletion events are established. How long each service was unavailable would require its own monitoring and logs; this report does not fill those gaps with estimates.

The records also limit attribution. A token identifier can associate operations, while source addresses and dashboard context can offer investigative leads. They do not identify the person responsible. Whether initial access involved a browser session, leaked credentials or an authorization process remains unresolved. We found no evidence sufficient to attribute this incident to a Cloudflare platform vulnerability.

## 6. Containment, revocation and removal were separate steps

At 11:29:38, after preserving the source and configuration evidence, we replaced the malicious Worker's code with a pass-through implementation:

```js
export default {
  fetch(request) {
    return fetch(request);
  },
};
```

This temporary measure kept the legitimate request path working while removing response rewriting and the special proxy behavior. The deployment credential available to us could update a Worker but could not manage account tokens or Zone routes; those API requests returned 403. Stopping the malicious code first bought time for the administrative steps.

At 11:45:36, the administrator deleted the target token under **Manage Account → Account API Tokens**. A subsequent audit read confirmed the matching successful `Delete Token` record and revocation event. We verified the specific token rather than removing unrelated build credentials with similar-looking names.

At 11:51:28, after checking that the contained Worker had no application dependencies, resource bindings or custom domains, we deleted it. The API reported success, the account's Worker inventory no longer included it, and the dashboard reported that the object did not exist. After refreshing, the administrator reported that the associated entry had disappeared. The available API credential still could not enumerate every Zone route, so this is not presented as a complete routing audit of the account.

<figure>
  <a href="/assets/incidents/2026-09-27-cloudflare/worker-deleted.png"><img src="/assets/incidents/2026-09-27-cloudflare/worker-deleted.png" alt="The Cloudflare dashboard reports that the Worker cf-w-d6b620c5 does not exist or has been deleted" width="1912" height="1312" loading="lazy" style="width:100%;height:auto;"></a>
  <figcaption>Figure 3. The original dashboard screenshot after deletion, not a reconstruction. It retains the Worker name and contains no API secret. It establishes that this Worker is gone, not that every account setting has been audited.</figcaption>
</figure>

## 7. What recovery verification established

Post-containment checks used Mac and Windows User-Agents against the Chinese, English and Japanese homepages, plus marketplace, flags and robots.txt: twelve HTTP requests in total. All returned 200 without the known injection indicators. Browser checks also found no previously observed hidden nodes or corresponding script errors. These results support recovery of the pages and endpoints tested, not a conclusion about every visitor's historical exposure.

Deployment was verified independently. A local `pnpm cf:deploy` completed, and a real GitHub push to the private repository's `dev` branch triggered a successful automatic build and deployment. Responding to the security incident did not replace acceptance of the original deployment fix.

The administrator subsequently selected **Full** SSL and enabled IPv6. Audit records show successful edits to those settings. DNS then returned AAAA records. An explicit connection to one of the resolved IPv6 addresses, bypassing the local HTTP proxy, returned HTTP 200 from Cortex with TLS certificate verification passing.

Full and Full (Strict) still differ: Full encrypts HTTPS connections to an origin but does not validate its certificate; Full (Strict) adds that validation and requires a suitable origin certificate.[3] Enabling IPv6 restores the corresponding connection capability.[4] **Neither setting revokes a malicious token or removes injection code.** Historical setting values could not be recovered through the available history interface. These are current-state checks, not a claim that every pre-incident setting has been restored.

At the time of writing, both Cortex deployment paths, the checked pages and a real IPv6 request were working. The other previously deleted application Workers were not individually restored as part of this Cortex repair. The initial account-access path also remains unknown. Those limits belong alongside the statement that the current service has recovered.

## 8. What I want to keep from this investigation

The useful habit was to be precise about what each successful check actually established.

A successful build establishes that its commands completed. The absence of a script from application source establishes something about that source. A valid certificate establishes something about a TLS connection. To understand what a visitor receives, we also need the public response and browser DOM, together with the routing, Workers and account permissions that can affect them.

For my own deployments, public-page inspection will remain part of acceptance. Account-side changes also need a traceable inventory. Temporary containment and final cleanup deserve separate entries: replacing code with a pass-through, withdrawing write access and removing unwanted resources solve different parts of the problem. A record should make an unfinished step visible.

This report does not end with every question answered. We located the response modification, associated the relevant operations with a token and verified Cortex's recovery. Initial access and historical visitor impact remain open investigative questions. Leaving those gaps visible makes the account more useful than a cleaner story would be.

## Sources and method

This report draws on build logs, API receipts, audit extracts, Worker source, HTML responses and dashboard screenshots retained during an investigation I carried out with Codex. The downloaded Worker was not replayed locally, and the remote payload referenced by its conditional branch was not retrieved or executed. Live-page observations and source-level capabilities are reported separately. The two diagrams are reconstructions; the deletion screenshot is an original record.

A [field-filtered timeline CSV](/assets/incidents/2026-09-27-cloudflare/timeline-extract.csv) is available for checking the sequence described here. It is a derived extract, not an untouched audit export or independent third-party forensic verification. Full account and Zone identifiers, token IDs, email addresses, operation-source IPs and raw request contents are omitted. API secrets, the complete malicious source and external payloads are not published with the article.

1. Cloudflare, [Custom Domains: interaction with Routes](https://developers.cloudflare.com/workers/configuration/routing/custom-domains/#interaction-with-routes). This documents the routing mechanism; it does not substitute for incident-specific evidence.
2. Cloudflare, [Account-owned API tokens](https://developers.cloudflare.com/fundamentals/api/get-started/account-owned-tokens/).
3. Cloudflare, [Full](https://developers.cloudflare.com/ssl/origin-configuration/ssl-modes/full/) and [Full (Strict)](https://developers.cloudflare.com/ssl/origin-configuration/ssl-modes/full-strict/).
4. Cloudflare, [IPv6 compatibility](https://developers.cloudflare.com/network/ipv6-compatibility/).

*First recorded on September 27, 2026. New findings about initial access or impact will be added as dated updates, rather than silently recast as facts known during the original investigation.*
