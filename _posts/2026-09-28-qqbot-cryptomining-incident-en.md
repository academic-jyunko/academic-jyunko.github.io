---
layout: post
title: "I meant to fix a QQ bot. A wall of migration threads led me to two cryptominers."
date: 2026-09-28 22:45:00 +0800
author: HsiangNianian
lang: en
permalink: /2026/09/28/qqbot-cryptomining-incident-en.html
toc: true
excerpt: "I was adding voice messages, web reading, and progress replies to Krypton. Then I checked the server. The bot was still learning to report progress; the server was already using more than a hundred cores to announce its side hustle."
description: "A cryptomining incident: a QQ bot feature request, disguised miners, separate SSH and RAGFlow intrusion trails, and the limits of containment and cleanup."
---

[中文版](/2026/09/28/qqbot-cryptomining-incident.html)

> Put together from that day's records. All times are Beijing time (UTC+8); account and infrastructure details are anonymized. The response status is as of that evening—the case isn't closed.

September 28, 2026.

I was adding features to a QQ bot.

Her name is Krypton; we usually call her Sha Ke—roughly "silly Krypton". The plan for the day: let her send voice messages, read web pages on her own, know how much budget she had left on a long task, and, when it made sense, say something first and then keep working.

Then I glanced at the server.

…why so many `migration` threads? Why is the CPU this high?

A few hours later I had two intrusion timelines, a quarantined bundle of miners, and this write-up, which I had absolutely no plans to write.

The bot still hadn't learned to report on her own. The server, meanwhile, had used more than a hundred cores to tell me about its side hustle.

## It really did start with a bot that couldn't send voice messages

The first error was a concrete one: sending a voice message returned `ActionFailed`. I found the bot's tmux window, followed the audio file locations and the send path, and it turned out the paths didn't match. Fixed that, and her voice message arrived in QQ.

Next, the way she talked. Whenever she wasn't sure, out came "I haven't verified this part yet, so I won't jump to conclusions." Once or twice, fine. Every time, and it was like a friend in the group chat suddenly changing into a customer-service uniform.

So the canned fallback became something generated from the conversation, then checked. The rule I set: if generating or checking fails, say nothing—don't paper over it with a template. (That rule matters later. Some of her silences were failure handling, not her deciding to ignore us.)

I also removed a leftover voice-diagnostics plugin. Its job was long finished, but it was still answering messages and trying to write into a temp directory that no longer existed. Deleting the plugin meant restarting the process too—the file on disk was gone, but the instance in memory wasn't going to leave on its own.

Then she got a tool called Bash, starting with nothing but a restricted `curl`: public HTTP/HTTPS, GET/HEAD, capped response size. Permissions stay with the host; one chat message can't promote it to an arbitrary shell.

She already had Tavily search. What I wanted to see: given a moderately complex task, would she combine search and page reading herself, rather than following an order we'd laid out for her?

In practice, the tools got called, but the results didn't get read well. Long pages were truncated; answers sprouted links with nothing behind them; a few rounds of rewriting still failed the check. The terminal was buzzing; the QQ side was as quiet as an empty office.

So in went text extraction, a per-conversation page cache, line-based reading and keyword lookup. The tool tells her how many lines there are and where the next chunk starts, and the check only counts passages actually returned to her. I mean—having the whole page in a cache isn't the same as the model having read it, and "I've read it all" doesn't get to paper over that.

Only then came budget and mid-task replies: she can see her remaining rounds, tool allowance, time and length limits, and she's allowed to send one checked progress message and keep going within the same budget.

I was particular about one thing here: *when* to speak is the model's call. The code supplies the environment and the limits—no script saying "three rounds left, say something". The path works. But on one real, complex task she never used it, and ended in silence anyway. So the ability is there; that she's learned to use it on her own is not shown.

I'm still working out how to make her more proactive. Two other residents of the server needed no such help.

## A wall of migration threads, doing almost nothing

Around 18:47 I turned to the process list.

The machine has 128 logical CPUs, and so 128 `migration/N` threads. They're the Linux kernel's per-CPU stopper threads, used for things like task migration—nothing to do with some app frantically migrating a database. The name matches the [kernel source](https://github.com/torvalds/linux/blob/master/kernel/stop_machine.c).

Over a three-second sample, that whole row of threads used about 0.3% of a single core. The ones that stood out were these two:

| Process name | Actual location | Sampled CPU |
| --- | --- | --- |
| `-bash` | An executable in a hidden directory on the host | About 9943%, i.e. 99 logical cores |
| `xmrig` | A hidden cache directory inside the RAGFlow container | About 2642%, i.e. 26 logical cores |

(One logical CPU counts as 100%, so a multithreaded process going past 100% is normal. To avoid mistaking a historical average for current load, I read the CPU-time deltas from `/proc`.)

The row of suspicious-sounding threads was basically idling, while the most respectable-sounding name of all—a shell—had nearly booked out the machine. Judge a server's health by process names and you'll suspect the wrong people.

That `-bash` wasn't where the system Bash lives at all; it was tucked under `/var/tmp/.ICE-Unix/`. Its static strings mentioned RandomX, stratum and XMRig, with a launcher, keepalive scripts and a PID file sitting next to it.

The other one was XMRig in the container, with pool configuration and watchdog scripts. [XMRig](https://xmrig.com/docs/miner) itself is open-source mining software; the problem was that someone had deployed it without permission and wired auto-restart into a service's startup entry.

One guess could be ruled out right away: the container miner had been running since September 21, and the host miner's files changed on September 25—both before the bot got `curl` on the 28th. "The new tool let the miners in" isn't supported by the evidence.

## Stop the miners first, then ask who invited them

Save first: process metadata, executables, configuration, crontabs, startup scripts, with hashes recorded. Then suspend the confirmed miners and watchdogs, and cut off the autostart paths found so far.

The first sample afterwards showed overall CPU busy at about 7.4%; at 19:04 it was about 6.3%. Instant relief—but that's only containment. How they got in, and whether they could come back, was still unanswered.

That's when I remembered two accounts had fairly simple passwords. I'll call them Account A and Account B. Both were in the `sudo` group, and SSH allowed password logins at the time.

A weak password is a lead, not a finding. Only after lining up the auth logs, file times and startup records did Account A's story come out in full.

## First trail: log in with the password, then change it

This is what the auth logs, syslog and file metadata agree on—all on September 25:

| Time | Record |
| --- | --- |
| 21:19:14 | External Source 1 logs in to Account A with a password |
| 21:19:15 | `.ssh` and `authorized_keys` change; a suspicious public key appears |
| 21:19:18 | Source 1 logs in again with a password |
| 21:19:21 | Logs record Account A's password being changed |
| 21:19:24 | Source 2 logs in to the same account with a password |
| 21:25:13 | Miner binary and startup scripts change; syslog records the crontab being replaced |
| 21:26:01 | Cron starts running the keepalive script in the hidden directory, once a minute from then on |

In the retained auth logs, Source 1 left 353,477 failed password attempts. Account A's only three successful SSH authentications were exactly those three password logins.

Log in, plant a key, change the password, drop the miner, add cron—about six minutes, start to finish. This is well past "the password was simple, so maybe it got guessed": Account A was brute-forced and then used to deploy a miner, and the evidence for that is strong.

Of course, file times aren't a per-command audit. Those timestamps can't be stretched into a full recording of the attacker's terminal.

Account B had logins too, but nothing ties them to those two sources. A similarly simple password isn't a reason to declare it compromised as well.

At 19:17, after backing up and checking the SSH config: new SSH logins blocked for Account A, the suspicious key quarantined, and Account B restricted to its existing keys. SSH reloaded, the management account reconnected fine, and running services weren't touched.

One thing I can't gloss over: Account A had password-protected sudo, and the attacker had already changed that password. No successful sudo shows up in the retained logs—but that doesn't prove privileges were never escalated.

## Second trail: a RAGFlow workflow called "System Diagnostics"

The container miner showed up earlier than the September 25 SSH intrusion, so I didn't rush to join the two into one story.

RAGFlow's Docker log was about 23.6 GB. I used the miner files' change times to narrow the windows, saved the relevant slices with offsets and hashes, and lined them up against Nginx access logs and database records.

Out came **six Canvas workflows carrying remote-command payloads, and five model configurations pointing at the same external server**. The earliest workflow, created in the small hours of September 14, was called `System Diagnostics`.

To be fair, the name wasn't wrong—it did end up diagnosing that this system was taking orders from someone other than its admin.

| Time | Record |
| --- | --- |
| September 14, 05:21:03 | An external source registers a regular RAGFlow user; the database has the matching account |
| 05:35:32 | An OpenAI-compatible model pointing at an external server is added |
| 05:35:34 | The `System Diagnostics` workflow is created |
| 05:35:46–05:35:48 | The app logs the rendered prompt; the access log records the run request completing |
| September 20, 05:56–05:57 | The malicious workflow runs again; files in the miner directory change at the same time |
| September 21, 02:45–02:46 | Another batch of registrations, model configs and workflow runs, right next to the miner landing in the cache directory |

A "regular user" here is an application account, not a Linux login. This route never needed an SSH password at all.

The workflows were shaped `Begin → PubMed → LLM → Message`. Custom citation rules in the LLM prompt went into `citation_prompt()`, where a plain `jinja2.Environment` rendered them as a template.

That's where it broke: a user-controlled template got past the boundary it should never have crossed, reaching Python objects and running system commands. It lines up with the already published [CVE-2026-45312 / GHSA-wpg4-h5g2-jxm6](https://github.com/infiniflow/ragflow/security/advisories/GHSA-wpg4-h5g2-jxm6)—the advisory's example uses DuckDuckGo, ours used PubMed, and both feed the same citation-template step.

So "the model got jailbroken" would be the wrong description. The dangerous part happened while the application rendered a template; it didn't depend on the model agreeing to run anything.

I didn't stop at the image tag, either. The saved `generator.py` has exactly the same SHA-256 as the official v0.23.1 file, and the app logs kept the rendered output—the original expression replaced by an output marker, and in another entry, a response from the result-callback endpoint.

That says much more than an HTTP 200: the malicious template really did reach execution. Unfortunately the command server's full responses weren't kept, so the exact commands that installed the miner can't be rebuilt one by one.

The external address in the template is the same one the miner's watchdog script downloads from, and several workflow runs sit right next to the miner files landing. Put together, the link between the RAGFlow template injection and the container miner is strong. The missing command contents are still missing, though—reasoning can't fill them in as if they were logs.

One more boundary: the container ran as root, and the service entry script was a writable mount, so the container could tamper with the matching file on the host. That alone proves neither a container escape nor host root.

At this point the two trails sit like this:

| | Host account trail | RAGFlow container trail |
| --- | --- | --- |
| Entry found | SSH password authentication | App registration, malicious workflows, template injection |
| Key date | September 25 | Workflow activity traced back to at least September 14 |
| Running as | Account A | Root inside the container |
| Link to mining | File changes, crontab replacement, cron execution | Rendered output, same external address, adjacent landing times |
| Can't conclude | That escalation is ruled out | That the container escaped |

Nor does anything show the two trails are the same attacker. They just happened to crowd onto the same server.

## "Did we find everything?"—and two more entry points appear

Finding two running miners is a long way from "nothing's left". I kept going: the real executable behind every process, cron, systemd, shell startup files, container entrypoints.

On the RAGFlow side, beyond `.bashrc`, cron and the service entry script, there was a `cache-warm.service` and a profile script that ran at login—both pointing at the same miner watchdog.

These were extra persistence hooks, not new running processes. Files on disk get their own count: in the end, three miner executables with different hashes, one of them not running at the time. The "two cryptominers" in the title are the two processes that were eating the CPU at the start.

Had I declared victory the moment the CPU came down, I'd have been patching my own incident report half an hour later.

Next to the host miner was a small tool for faking process names. A copy of the launcher was only unpacked in an isolated environment, and the script inside clearly calls that tool to make the miner show up as `-bash`. The sample was never run.

The script also picks out other high-CPU processes and tries to kill them. Come to take the CPU, and send everyone else home while you're at it. Still, having that capability doesn't prove any particular service exit was its doing, and what an ordinary account can kill is limited by permissions anyway.

Some innocent programs got suspected along the way, too. A few Node processes launched from `/tmp` matched the official release binaries hash for hash. And one genuine database migration came from Coze's container init config. After all that, a migration that actually was a migration—shame it had nothing to do with that row of threads.

Those distinctions are worth keeping. An ops tool that translates "I don't recognize this" into "delete it" can do as much damage as the miners.

## Before stopping RAGFlow, ask who else its Elasticsearch works for

I said: if nothing else depends on it, stop RAGFlow. It turned out the app and its MySQL and MinIO had no other users—but the Elasticsearch next to it was shared by an asset library, a knowledge base and another Q&A service.

Which Compose project a container is named under and who it actually serves today are two different things. Stopping the whole project would have taken unrelated services down with it.

At 19:59: the RAGFlow app container and its own MySQL and MinIO stopped, auto-restart disabled, and those services isolated in Compose. The shared Elasticsearch and Kibana stayed up, and the database and object-storage volumes were kept.

Then I checked listening ports, service responses and index counts: the shared services hadn't restarted, and everything else behaved as before. Elasticsearch was yellow going in and yellow coming out—I didn't "accidentally fix" it.

That closes the confirmed application entry point. But any shared credentials the compromised app could have read don't expire just because the container stopped; rotating them and checking the shared data still has to happen.

## The miners clocked out, into an archive you shouldn't double-click

The confirmed miners, launchers, keepalive scripts and the infected container's writable layer were archived locally. Files are stored by content hash, with a manifest recording original paths, permissions and times; after downloading, both the archive and each content object were hash-checked.

About 18.8 MB: 115 unique content objects, corresponding to 138 regular-file records. The directory is access-restricted, the archive is read-only and carries the OS quarantine flag. They're still malware, mind—the flag only guards against slips of the hand; it doesn't disinfect anything.

At 20:09, with local verification done: the still-suspended host miner terminated, the confirmed miner directories and sample copies on the server deleted, the malicious crontab removed, and the infected RAGFlow app container deleted. It wasn't restarted for the cleanup, and the data volumes weren't touched.

The review also corrected one note: the host cron job had earlier been disabled only because its startup script was renamed—the cron line itself was still there. After confirming the account had no other scheduled jobs, the malicious crontab was removed entirely.

At 20:11, the old processes, watchdogs, original files and the original container were all gone. Re-checking selected directories, current executables and the forensic copies on the server turned up no hits on known miner names or hashes.

That result has a scope, though: some dependency caches were skipped, depth was limited, and the data disk wasn't read byte by byte. "No hits this pass" can't be written up as "all malicious files are gone".

The stopped database still holds the malicious workflow records. Bringing RAGFlow back can't mean starting the old service and calling it a trusted environment again.

Account A couldn't simply be deleted either. It still runs three business projects, with Python and Node environments in its home directory. So the account and services stay, new SSH logins stay blocked; retiring the account waits on moving those services, and the local password, sudo rights and old sessions still need finishing off.

All day, the things whose names sounded easiest to deal with were the ones whose dependencies objected the most.

## The code was fixed long ago; the advisory still says "no patched version"

I've contributed to the GitHub Advisory Database before, so I thought I'd see whether this material could add some analysis or fix verification.

Checking the public record first turned up a very specific gap: at the time of writing, the project advisory's patched-version field was empty. Yet [PR #14068](https://github.com/infiniflow/ragflow/pull/14068) was merged back on April 13, switching the rendering environment to `SandboxedEnvironment`, and [v0.25.0](https://github.com/infiniflow/ragflow/releases/tag/v0.25.0), released April 21, already includes it.

The [commit ancestry](https://github.com/infiniflow/ragflow/compare/6fdca2d2125edcf9bc91eeb26975314483e9e892...v0.25.0) checks out too. The [official CVE record](https://github.com/CVEProject/cvelistV5/blob/main/cves/2026/45xxx/CVE-2026-45312.json) for the same vulnerability already lists CWE-1336, while the matching field in the project advisory is empty.

To confirm the known boundary, I pulled the real environment declarations and `citation_prompt()` out of several versions and ran an attribute-access check that executes no commands, reads no files and makes no network calls:

| Source version | Ordinary template rendering | Known object-attribute traversal |
| --- | --- | --- |
| v0.23.1 | Passes | Allowed |
| v0.24.0 | Passes | Allowed |
| v0.25.0 | Passes | `SecurityError` |
| v0.27.2 | Passes | `SecurityError` |
| main snapshot pinned that day ([`d64b84c`](https://github.com/infiniflow/ragflow/commit/d64b84c7095b1edb81987bbf82cdc95b36a28cbf)) | Passes | `SecurityError` |

This only checks the extracted functions—no full application, no attack replayed over HTTP. It supports "this historical fix works"; it can't vouch for a whole release, every workflow, or other template bugs.

At 21:12 I opened [issue #20324](https://github.com/infiniflow/ragflow/issues/20324), asking for the patched version, the actual fix reference and the CWE to be filled in, and asking the maintainers to consider analyst or fix-verification credit.

To be clear: it's an advisory correction request. I didn't discover the vulnerability and didn't write the fix; re-submitting an existing patch doesn't turn into a new security contribution. Whether credit fits is the maintainers' call.

The incident material might support a fuller anonymized analysis later, but this post includes no raw logs, malicious workflows, attack commands or miner samples. Deeper workflow-level verification isn't finished, so it doesn't go in the "verified" column yet.

## When can I say it's cleaned up?

What I can confirm: the known miners and watchdogs are terminated, the confirmed samples and persistence hooks are removed from the server, the infected app container is deleted, and the anomalous SSH entry is restricted. The direct cause of the maxed-out CPU is dealt with.

What I can't confirm yet: whether there were earlier intrusions, host privilege escalation or lateral movement, whether shared credentials leaked, or whether other data was read or taken. Actually trusting the system again needs credential rotation, migrating or rebuilding services, checking the remaining entry points, and time spent watching.

What stuck with me most were all the places where I had to stop and tell two things apart: a process name and the actual program, an account and the person it belongs to, template execution and model behavior, root in a container and root on the host—and the CPU coming down versus the environment being trustworthy again.

I only wanted the bot to stop going quiet during long tasks. Whether she's learned when to speak up, I'll have to keep watching; the server's side business, at least, is shut down.

How much of the feature request got done, let's not get into. But "autonomous emergence" in the scope of the request—that, I certainly got to experience.

---

> **Update (2026-10-28):** Issue #20324 got a reply from a maintainer on September 30, thanking me for the verification. As of today, the patched-version and CWE fields of GHSA-wpg4-h5g2-jxm6 are still empty.
