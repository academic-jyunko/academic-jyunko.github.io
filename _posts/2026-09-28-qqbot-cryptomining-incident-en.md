---
layout: post
title: "I meant to fix a QQ bot. A wall of migration threads led me to two cryptominers."
date: 2026-09-28 22:45:00 +0800
author: HsiangNianian
lang: en
permalink: /2026/09/28/qqbot-cryptomining-incident-en.html
toc: true
excerpt: "I was adding voice messages, web reading, and progress replies to Krypton. Then I checked the server. The bot was still learning to report progress; the server was already using more than a hundred cores to announce its side hustle."
description: "Investigating a cryptomining incident with Codex: a QQ bot feature request, disguised miners, separate SSH and RAGFlow intrusion trails, and the limits of containment and cleanup."
---

[中文版](/2026/09/28/qqbot-cryptomining-incident.html)

On September 28, 2026, I meant to add a few features to a QQ bot.

Her name is Krypton. We usually call her Sha Ke, an affectionate Chinese nickname roughly meaning “silly Krypton.” The plan was to get voice messages working, let her read web pages, and show her how much budget she had left during a long task. If useful, she could send a reply along the way and then keep working.

Then I glanced at the server. Why were there so many `migration` threads, and why was CPU usage so high?

A few hours later, the deliverables included two intrusion timelines, a quarantined archive of mining software, and an incident report I had never intended to write.

The bot was still learning to report progress. The server was already using more than a hundred cores to announce its side hustle.

I worked through this with Codex again. This account is based on the records we collected that day. All times are Beijing time, UTC+8. Account and infrastructure details have been anonymized. The response status is as of that evening, and the investigation remains open. I want to keep what we saw at the time, what we established later, and what we still do not know separate.

## It really did start with a bot that could not send voice messages

The first error was specific: sending a voice message returned `ActionFailed`. We found the bot's tmux window and traced the audio file locations and sending path. After fixing a path mismatch, I received the voice message in QQ.

Then there was the way she talked. Whenever she was uncertain, she would fall back to something like, “I haven't verified this yet, so I won't jump to conclusions.” Once or twice was fine. Repeated often enough, it felt like a friend in the group chat had suddenly put on a customer service uniform.

We replaced the canned fallback with a response generated from the conversation, followed by a validation step. I chose a clear boundary: if generation or validation failed, send nothing rather than fill the gap with a template. That mattered later. Some silence came from failure handling, not from a decision to ignore us.

We also removed a leftover voice diagnostics plugin. Its job was done, but it was still handling messages and trying to write into a temporary directory that no longer existed. Removing the file required a process restart too. The instance in memory had not received the retirement memo.

Next came a tool called Bash, initially limited to a constrained form of `curl`: public HTTP/HTTPS destinations, GET/HEAD requests, and bounded response sizes. The host controlled the permissions. A chat message could not promote it into an unrestricted shell.

She already had Tavily search. I wanted to see whether she would combine search and page retrieval on a moderately complex task, without us prescribing the sequence first.

In practice, calling a tool and reading its result well turned out to be different skills. Long pages were truncated. Answers included links the evidence did not support. Several revisions could still fail validation. The terminal looked busy; the QQ conversation looked abandoned.

So we added text extraction, a page cache for the conversation, line-based reading, and keyword search. The tools reported the total line count and where to read next. Validation used only the passages actually returned to the model. Having the whole page in a cache did not mean she had read it, however confidently she said she had.

Finally, we exposed the remaining model rounds, tool allowance, time, and length limits. We also added a way for her to send a validated progress message and continue within the original budget.

I specifically wanted the model to choose when to use it. The code would supply the tools and constraints, without a fixed script saying “announce progress when only a few rounds remain.” The capability worked, but on one real complex task she did not choose to use it and still ended in silence. We had implemented the option; we had not demonstrated reliable independent use.

I was still studying how to make her more proactive. Two other residents of the server had already mastered that part.

## The wall of migration threads was mostly minding its own business

At about 18:47, we turned to the process list.

The machine had 128 logical CPUs and 128 `migration/N` threads. These belong to the Linux kernel's per-CPU stopper mechanism, used for work including task migration. They were not an application frantically migrating its database. The name matches the [kernel source](https://github.com/torvalds/linux/blob/master/kernel/stop_machine.c).

Over a three-second sample, those threads together used about 0.3% of one CPU. Two other processes were doing most of the work:

| Process name | Actual location | Sampled CPU use |
| --- | --- | --- |
| `-bash` | An executable in a hidden directory on the host | About 9943%, equivalent to 99 logical cores |
| `xmrig` | A hidden cache directory inside the RAGFlow container | About 2642%, equivalent to 26 logical cores |

These percentages count one logical CPU as 100%, so a multithreaded process can exceed 100%. We also measured changes in CPU time from `/proc`, rather than treating a historical average as current usage.

The suspicious-looking crowd of threads was barely working. The process with a perfectly respectable shell name had almost booked out the machine. Process names were proving to be poor character references.

The executable behind `-bash` was not the system Bash binary. It was hidden under `/var/tmp/.ICE-Unix/`. Static inspection found strings associated with RandomX, stratum, and XMRig, alongside launchers, keepalive scripts, and PID files.

The other process was XMRig inside the container, with mining pool settings and watchdog scripts. [XMRig](https://xmrig.com/docs/miner) is open-source mining software. The problem here was its unauthorized deployment and the extra startup hooks inserted into a business service.

The timing also ruled against one tempting explanation. The container miner's process had started on September 21, and the host miner's files had changed on September 25. Both predated the bot's new `curl` capability on September 28. The evidence did not support blaming that feature for letting the miners in.

## Stop the mining first, then find out who invited it

We saved process metadata, executables, configuration, crontabs, and startup scripts, and recorded their hashes. Then we suspended the confirmed miners and watchdogs and blocked the automatic startup paths we had identified.

The first sample afterward showed overall CPU utilization at about 7.4%. At 19:04, it was about 6.3%. That was an immediate improvement, but it was still containment. We had not answered how the attackers got in or whether they could return.

I mentioned that two accounts had weak passwords. I will call them Account A and Account B rather than publish their actual identities. Both belonged to the `sudo` group, and SSH allowed password authentication at the time.

A weak password was a lead, not a finding. Authentication logs, file timestamps, and startup records were what gave us a clearer picture of Account A.

## First trail: log in with a password, then change it

This is the sequence we could align across the preserved authentication logs, syslog, and file metadata. Every entry in this table is from September 25:

| Time | Record |
| --- | --- |
| 21:19:14 | External Source 1 successfully authenticated to Account A with a password |
| 21:19:15 | `.ssh` and `authorized_keys` changed; a suspicious public key appeared |
| 21:19:18 | Source 1 logged in again using a password |
| 21:19:21 | The logs recorded a password change for Account A |
| 21:19:24 | Source 2 logged in to the same account using a password |
| 21:25:13 | Miner binaries and startup scripts changed; syslog recorded a crontab replacement |
| 21:26:01 | Cron began running the keepalive script in the hidden directory, repeating every minute |

Within the retained authentication logs, Source 1 accounted for 353,477 failed password attempts. Account A had only three successful SSH authentications in that period: the three password logins in the table.

Login, public key, password change, miner files, cron. Roughly six minutes. This went well beyond “the password was weak, so someone might have guessed it.” The evidence strongly supported a password-based compromise of Account A followed by miner deployment.

File timestamps still are not a command audit. We could not turn those points into a complete recording of the attacker's terminal.

Account B had login records too, but no evidence connected them to those two sources. A similarly weak password was not grounds to label it a second compromised account.

At 19:17 on the response day, September 28, we backed up and checked the SSH configuration, blocked new SSH logins for Account A, and quarantined the suspicious public key. Account B was restricted to its existing public keys. We reloaded SSH, successfully reconnected with the management account, and left existing services running.

One limitation mattered: Account A had password-protected sudo access, and the attacker had already changed its password. We found no successful sudo use in the retained logs. That did not prove privilege escalation had never happened.

## Second trail: a RAGFlow workflow called “System Diagnostics”

The container's miner appeared before the September 25 SSH intrusion. We kept the two trails separate instead of rushing to join them.

RAGFlow's Docker log was about 23.6 GB. We located time windows around changes to the mining files, preserved the relevant sections with offsets and hashes, and compared them with Nginx access logs and database records.

That turned up **six Canvas workflows containing remote command payloads and five model configurations pointing to the same external server**. The earliest workflow, created in the early hours of September 14, was called `System Diagnostics`.

The name was not entirely wrong. It eventually helped me diagnose that the system was taking instructions from someone besides its administrator.

| Time | Evidence collected |
| --- | --- |
| September 14, 05:21:03 | An external source registered a regular RAGFlow user; the database contained the corresponding account |
| 05:35:32 | An OpenAI-compatible model pointing to an external server was added |
| 05:35:34 | The `System Diagnostics` workflow was created |
| 05:35:46–05:35:48 | The application logged the rendered prompt; the access log recorded completion of the workflow run request |
| September 20, 05:56–05:57 | Another malicious workflow run coincided with changes to files in a mining directory |
| September 21, 02:45–02:46 | Further registration, model configuration, and workflow execution records were close in time to miner files appearing in the cache directory |

A regular user here meant an application account, not a Linux login. This route did not require an SSH password first.

The workflows we found followed `Begin → PubMed → LLM → Message`. Custom citation instructions from the LLM prompt reached `citation_prompt()`, where an ordinary `jinja2.Environment` rendered them as a template.

That was the failed boundary. A user-controlled template could access Python objects and execute system commands. It closely matched the already disclosed [CVE-2026-45312 / GHSA-wpg4-h5g2-jxm6](https://github.com/infiniflow/ragflow/security/advisories/GHSA-wpg4-h5g2-jxm6). The published example used DuckDuckGo; our evidence involved PubMed. Both fed into the same citation-template rendering step.

Calling this a model jailbreak would miss the mechanism. The dangerous operation happened while the application rendered a template. It did not depend on the model agreeing to run a command.

We did not rely on the image tag alone. The preserved `generator.py` had exactly the same SHA-256 as the official v0.23.1 file. Application logs also contained rendered template output: the original expression had been replaced by an output marker, and another entry included a response from the result-reporting endpoint.

That was stronger evidence than an HTTP 200: the malicious template had reached execution. But the command server's complete responses were not retained, so we still could not reconstruct every command used to install the miner.

The external address in the template matched the download address in the mining watchdog script. Several workflow runs also fell close to the times the mining files appeared. Together, these were strong evidence linking the RAGFlow template injection to mining inside the container. The missing command contents remained a gap; inference could not turn them into recorded evidence.

There was another boundary to preserve. The container ran as root, and the service entrypoint was a writable bind mount. That allowed it to alter the corresponding file on the host. It did not, by itself, prove container escape or host root access.

At this point, the two trails looked like this:

| | Host account trail | RAGFlow container trail |
| --- | --- | --- |
| Identified entry | SSH password authentication | Application registration, malicious workflows, template injection |
| Key date | September 25 | Workflow activity traced back to at least September 14 |
| Execution identity | Account A | Root inside the container |
| Link to mining | File changes, crontab replacement, and cron execution records | Rendered output, matching external address, and closely timed file creation |
| What this does not establish | That privilege escalation was ruled out | That a container escape occurred |

Nor did the evidence establish that both trails belonged to one attacker. Sharing a server did not mean sharing an employer.

## “Have we found everything?” produced two more startup hooks

Finding two running miners was a long way from establishing that nothing else remained. We continued inspecting actual process executables, cron, systemd, shell startup files, and container entrypoints.

In RAGFlow, beyond `.bashrc`, cron, and the service entrypoint, we found `cache-warm.service` and a profile script that ran when a login shell started. Both pointed to the mining watchdog already identified.

Those were additional persistence mechanisms, not two more running miners. Files needed their own count too: we eventually confirmed three miner executables with different hashes, one of which was not running when examined. The “two cryptominers” in the title refers to the two processes first found consuming the CPU.

Had I declared the cleanup complete when CPU usage dropped, I would have needed to patch my own incident report half an hour later.

A process-name disguise tool sat beside the host miner. We unpacked a copy of the launcher in an isolated environment without executing the sample. The resulting script explicitly invoked that tool to make the miner appear as `-bash`.

The script also selected other high-CPU processes and attempted to kill them. Occupying the machine apparently was not enough; it wanted to send the other services home. But finding that capability did not prove it caused any particular service outage. An ordinary account's ability to terminate other processes is also limited by permissions.

Some legitimate programs looked odd too. Several Node processes started from `/tmp`; their binaries matched files from the official release. A real database migration came from Coze's container initialization configuration. After all that, we finally found a migration doing what its name suggested. It just had nothing to do with the original wall of kernel threads.

Those distinctions matter. An operations tool that translates “I don't recognize this” into “safe to delete” can do quite enough damage on its own.

## Before stopping RAGFlow, find out who else its Elasticsearch serves

I asked to stop RAGFlow if no other services depended on it. We found no other business services using the application itself, its MySQL, or its MinIO. The neighboring Elasticsearch instance, however, was shared by an asset library, a knowledge base, and another question-answering service.

A container's Compose project name did not tell us who depended on it now. Taking down the whole stack would have taken unrelated services with it.

At 19:59, we stopped the RAGFlow application container and its dedicated MySQL and MinIO, disabled automatic restarts, and isolated those services in Compose. Shared Elasticsearch and Kibana stayed up. Database and object-storage volumes were retained.

We then checked listening ports, service responses, and index counts. The shared services had not restarted, and existing applications behaved as they had before. Elasticsearch had been yellow before the intervention and remained yellow afterward. We had not accidentally fixed its health status along the way.

This closed the identified application entry point. It did not revoke shared credentials the compromised application might have read. Credential rotation and checks on shared data still needed follow-through.

## The miners clocked out and went into an archive best left unopened

We archived the confirmed miners, launchers, keepalive scripts, and the compromised container's writable layer locally. File contents were stored by hash, with a separate manifest recording original paths, permissions, and timestamps. After downloading, we verified both the archive hash and every content object's hash.

The archive was about 18.8 MB: 115 unique content objects corresponding to 138 regular-file records. Its directory had restricted access; the archive was read-only and marked for quarantine by the operating system. These were still malicious samples. A quarantine flag reduced the chance of a mistake; it did not disinfect them.

At 20:09, after local verification, we terminated the still-suspended host miner, removed the confirmed mining directories and sample copies from the server, removed the malicious crontab, and deleted the compromised RAGFlow application container. We did not restart it to perform cleanup, and we did not delete the business data volumes.

The review also corrected an earlier response note. The host's cron job had initially been disabled in practice by renaming its startup script; the cron entry itself was still there. This time, after confirming the account had no other scheduled jobs, we removed the malicious crontab completely.

At 20:11, the old processes, watchdogs, original files, and application container were gone. Checks of selected directories, current executables, and forensic copies left on the server found no matches for known miner names or hashes.

That result had a defined scope. Some dependency caches were skipped, search depth was limited, and we had not examined the entire data disk byte by byte. “No matches in this pass” did not mean “all malicious files ruled out.”

Malicious workflow records also remained in the stopped database. Restoring RAGFlow could not simply mean starting the old services and declaring the environment trustworthy again.

Account A could not be casually deleted either. It still ran three business projects, with Python and Node environments in its home directory. We kept the account and services while restricting new SSH logins. Retirement would require moving those services first; the local password, sudo privileges, and existing sessions still needed attention.

That day's recurring pattern was that anything which sounded ready to remove turned out to have dependencies willing to argue the point.

## The code was already fixed. The advisory still listed no patched version.

I had contributed to the GitHub Advisory Database before, so I wanted to see whether this material could support useful analysis or remediation verification.

Checking public information revealed a specific omission. At the time of writing, the project advisory still had no patched version recorded. Yet [PR #14068](https://github.com/infiniflow/ragflow/pull/14068) had been merged on April 13, replacing the relevant rendering environment with `SandboxedEnvironment`. The fix was included in [v0.25.0](https://github.com/infiniflow/ragflow/releases/tag/v0.25.0), released on April 21.

We also checked the [commit ancestry](https://github.com/infiniflow/ragflow/compare/6fdca2d2125edcf9bc91eeb26975314483e9e892...v0.25.0). The [official CVE record](https://github.com/CVEProject/cvelistV5/blob/main/cves/2026/45xxx/CVE-2026-45312.json) already listed CWE-1336, while the corresponding field in the project advisory was empty.

To check the known boundary, we extracted the actual environment declarations and `citation_prompt()` from several versions. The check accessed an object attribute without executing system commands, reading files, or making network requests:

| Source version | Ordinary template rendering | Known object-attribute traversal check |
| --- | --- | --- |
| v0.23.1 | Passed | Allowed |
| v0.24.0 | Passed | Allowed |
| v0.25.0 | Passed | `SecurityError` |
| v0.27.2 | Passed | `SecurityError` |
| Pinned main snapshot from that day | Passed | `SecurityError` |

This was a boundary check on extracted functions. We did not start the full application or replay an attack over HTTP. It supported the conclusion about this historical fix; it was not a security guarantee for an entire release, every workflow, or other template vulnerabilities. The main snapshot was pinned to [d64b84c](https://github.com/infiniflow/ragflow/commit/d64b84c7095b1edb81987bbf82cdc95b36a28cbf).

At 21:12, I opened [issue #20324](https://github.com/infiniflow/ragflow/issues/20324), requesting the patched version, actual fix reference, and CWE, and asking the maintainers to consider analyst or remediation-verifier credit. At publication, the issue remained open without a reply, and the advisory had not yet been updated.

It was an advisory correction request. We had not discovered the vulnerability, and we had not written the fix. Submitting an existing patch again would not make it a new security contribution. Credit was for the maintainers to assess too.

The incident material may support a more complete, anonymized analysis later. This article includes no raw logs, malicious workflows, attack commands, or miner samples. More extensive workflow-level validation was still unfinished; it did not belong in the “verified” column yet.

## When can we call it cleaned up?

We could confirm that the known miners and watchdogs had been terminated, identified samples and persistence mechanisms had been removed from the server, the compromised application container had been deleted, and the anomalous SSH entry point had been restricted. The immediate cause of the high CPU usage had been addressed.

We could not yet establish whether there had been earlier intrusions, host privilege escalation, lateral movement, exposure of shared credentials, or other data access or exfiltration. Restoring trust still required credential rotation, service migration or rebuilding, checks for remaining entry points, and continued observation.

What stayed with me was how often we had to stop and separate things that looked interchangeable: a process name and its executable, an account and its owner, template execution and model behavior, container root and host root, lower CPU usage and a trustworthy system.

I had only wanted the bot to stop going silent during long tasks. Whether she has learned when to speak up still needs observation. At least the server's side hustle has stopped.

I cannot claim the same success for every feature. But if the goal was to observe autonomous behavior, the scope of the task certainly developed a mind of its own.
