# Task 00022 Evidence Manifest

Date: 2026-07-27

This manifest indexes the current corrective-pass review artifacts. `QA product` and `QA native` entries use the build-gated deterministic authority and display its non-production qualification. `Production native` entries come from the non-QA `.app`; the credential captures predate Task 00022A and are qualified below. `r013 reference` entries are comparison inputs, not implementation evidence.

## Native Light/Dark Pairs

| File | Qualification | Dimensions | SHA-256 |
| --- | --- | --- | --- |
| `.agents/resources/native/task-00022-production-native-keychain-recovery-light.jpeg` | production native, Light | 1100x761 | `d9257b3621331f18e4241046ec891afe6e3db69513b45ca13fe0e64c1e0b6dcb` |
| `.agents/resources/native/task-00022-production-native-keychain-recovery.jpeg` | production native, Dark | 1100x761 | `f52b7b2c402defd58c2490c4b21fffe500698894baea23892b9bea4fbe2670cb` |
| `.agents/resources/native/task-00022-qa-native-workspace-start-light.jpeg` | QA native, Light | 1100x761 | `e80abe97e1cf06a401ef2d53ac0a718a4da5803bc6767410ba4b78c841b0ac42` |
| `.agents/resources/native/task-00022-qa-native-workspace-start.jpeg` | QA native, Dark | 1100x761 | `9eabb528240c94cb09219903c51b629a92117c6c29da68bf8c7c7c7f0f980a19` |
| `.agents/resources/native/task-00022-qa-native-chat-light.jpeg` | QA native, Light | 1100x761 | `2662fc7c75f4e90406e42898b18d45a892d8725e0fc874c6982fc8a1edf2554b` |
| `.agents/resources/native/task-00022-qa-native-chat.jpeg` | QA native, Dark | 1100x761 | `97adb70070ff2e62fc36a00d490a4ecefc91e5c1dbf018a0fb6bba214653a09b` |
| `.agents/resources/native/task-00022-qa-native-settings-light.jpeg` | QA native, Light | 1100x761 | `a8f1f2a011a03957ac94df34606ae72d6f658f115b51ad32bef3eb04a4d8f4fd` |
| `.agents/resources/native/task-00022-qa-native-settings.jpeg` | QA native, Dark | 1100x761 | `c2d65e369dfc16d4dc5c96eb14b17a4af3c68d0084d244f1d867ab0e86e4f354` |

The native appearance was changed through macOS System Settings while the QA `.app` remained open. The Settings route and exact Chat conflict/draft state survived the live Light-to-Dark change and Settings Back transition. System appearance was restored to Dark. Text-scaling containment is covered by component and Playwright checks; this record does not claim a separate native text-size toggle walkthrough.

## Credential And Approval Paths

| File | Qualification | Dimensions | SHA-256 |
| --- | --- | --- | --- |
| `.agents/resources/native/task-00022-production-native-keychain-success.jpeg` | production native, pre-Task22A credential Test success; layout superseded | 1100x761 | `2bd4fafc00592129c64f3ba05058563c490fc0f8a7a737ac37107128508f770f` |
| `.agents/resources/native/task-00022-production-native-keychain-persisted.jpeg` | production native, Continue reached Workspace Start | 1100x761 | `8e7a946d371c3db957df45c19a0b6d28d350efe4877888d02758a9f5063d8c76` |
| `.agents/resources/native/task-00022-production-native-keychain-readback.jpeg` | production native, post-restart Provider Settings read-back | 1100x761 | `a87ad494d9754225826b0e9464523c2e04193508a02198dca2ec349d39aaeac3` |
| `.agents/resources/native/task-00022-qa-native-provider-explicit-ask.jpeg` | QA native, explicit `Ask` Provider Test | 1100x761 | `b786717164d62e6fc01abfeea26470aee5260f0e0cdad3e7218906c290db8d04` |
| `.agents/resources/native/task-00022-qa-native-runtime-credential-approval.jpeg` | QA native, production-composed runtime credential approval | 1100x761 | `79a4a7a19a1186ba00162c7102e6a19813648d9dddebe4bd566bfa1c695d0e4d` |
| `output/playwright/task-00022-provider-explicit-ask.png` | QA product, explicit `Ask` Provider Test | 1280x799 | `c03cc00d4b413c68b5b287cc53414c25127840aedad807a56826e54bdb93cf6b` |
| `output/playwright/task-00022-runtime-credential-approval.png` | QA product, runtime credential approval | 1280x1718 | `585a87d5a443ea5834cd4019a8f266707462bf2d77ab5144c801ef854bdf6c74` |

The three production-native captures use a fresh mode-0700 acceptance home, a unique disposable Keychain service, a dummy credential, and a localhost-only HTTP fixture. One Test produced one production-ready model, one Continue reached Workspace Start without a C4OS approval, and a controlled restart reopened Workspace Start before Settings read back the persisted provider. Non-secret Keychain metadata confirmed the installation-key item, and scans of the isolated vault/configuration/database found no plaintext credential. The item, home, app, and fixture were removed after capture. These captures predate Task 00022A and support only the credential/persistence boundary; the Test-success image visibly contains the now-removed model panel and is not current layout evidence.

The explicit-`Ask` Provider Test pauses before contacting the provider and before persisting the draft; the native capture shows the credential field cleared while Deny and Allow once remain available. The runtime capture mounts the production approval center and states that OpenCode requests temporary credential use while credential bytes stay inside the provider boundary and never enter renderer state or runtime arguments. Both native surfaces display the deterministic-QA qualification.

## First Run And Workspace Start

| File | Qualification | Dimensions | SHA-256 |
| --- | --- | --- | --- |
| `output/playwright/task-00022-onboarding-wide.png` | QA product, blank first run | 1440x900 | `2bdb03cabb2904bba725e181c4ce8df38709efdd7be5571518f9c7bae0794a1f` |
| `output/playwright/task-00022-onboarding-tested-wide.png` | QA product, successful Test, no picker | 1440x900 | `b7f3a8d67c735298c5e263c28f8a980ad8c1401f0126c417f051ade7658e6c26` |
| `output/playwright/task-00022-onboarding-tested-narrow.png` | QA product, successful Test, no picker | 390x844 | `e6d678d37465d4cca77fff9df237f67cbc964de0143751be4d07859c0dfe596e` |
| `output/playwright/task-00022-onboarding-failed-wide.png` | QA product, failed Test | 1440x900 | `65d09e2ea8e95697597979da5167a45c8aaebff7cf31ab47300169d2f9897933` |
| `output/playwright/task-00022-onboarding-failed-narrow.png` | QA product, failed Test | 390x844 | `7418f4bc52cdada4d49191ddd61ee61780cdcec1020da8db0be439f7df080690` |
| `output/playwright/task-00022-onboarding-no-model-wide.png` | QA product, zero usable models | 1440x900 | `14ba7ac47f9ebc669234e4f36a999e34fbb6d0eb345aa4c2125524b7dbd41523` |
| `output/playwright/task-00022-onboarding-no-model-narrow.png` | QA product, zero usable models | 390x844 | `d568585398c461d6d027534c1ae7f37cdab8a919e9aa79a6285704383c575ff8` |
| `output/playwright/task-00022-workspace-start-wide.png` | QA product | 1440x900 | `3d48021078bb9aa9a9a63eae20f84db509322b4c02e357f834e58850441ccb83` |
| `output/playwright/task-00022-first-run-journey.webm` | QA product recording, 1440x900 viewport, 379,734 bytes | video | `d5edd87a7045b0886eb8cd5fab7dcd96d019728dcb28ffa6bd42a4bf223ea569` |

The refreshed recording follows blank onboarding through successful Test with no model/default panel, Continue, Workspace Start, and entry to Chat. The pre-Task22A production-native credential captures above cover the normal Keychain create/read/restart path and exceptional secure-storage recovery path only; they are not current onboarding-geometry evidence.

## Settings Matrix

| File | Qualification | Dimensions | SHA-256 |
| --- | --- | --- | --- |
| `output/playwright/task-00022-settings-providers-wide.png` | QA product | 1440x900 | `f72c768894f4ffdbd4118a24999bb90a13989e18f24d850bd089b8ee0b2ddbc7` |
| `output/playwright/task-00022-settings-providers-narrow.png` | QA product | 700x840 | `e045974319e31618f3a8273723f24043212b3b00f4fb95e52b012e7f477ae067` |
| `output/playwright/task-00022-settings-models-wide.png` | QA product | 1440x900 | `f820c483d6b72f5dd675c1e46a746ec1e12976d09437d091747c5d00bfaee971` |
| `output/playwright/task-00022-settings-models-narrow.png` | QA product | 700x840 | `f23f1d86d62a832b488e27160ae7d3a421c74625c56140b4be94f39fef5049b8` |
| `output/playwright/task-00022-settings-runtimes-wide.png` | QA product | 1440x900 | `1a80ac8eeaeea92b08adf95a19cd0eef2e06f5002e27a1e74f5454c582a862f1` |
| `output/playwright/task-00022-settings-runtimes-narrow.png` | QA product | 700x840 | `116135c692aec44cb8feea0c07c01ca75d97362871c69f273030d93ca3ca0045` |
| `output/playwright/task-00022-settings-configuration-wide.png` | QA product | 1440x900 | `e489105468fa946f0c63a2fb9ac050b1b1635c94cff35d7dd7d11974bda84ac8` |
| `output/playwright/task-00022-settings-configuration-narrow.png` | QA product | 700x840 | `a3463f2f71586521e57e0816023666a5ab899eceba51d3b01953eded6fd6c90e` |
| `output/playwright/task-00022-settings-advanced-policies-wide.png` | QA product | 1440x900 | `c9e547192a68f346f71ba1c611a89be10bb72e4a1b5397e001699b3257fba953` |
| `output/playwright/task-00022-settings-advanced-policies-narrow.png` | QA product | 700x840 | `74f6076d5eecc8ded98a0bc64479a898b595e99d9736fdff5be03e406de1d5de` |
| `output/playwright/task-00022-settings-plugins-wide.png` | QA product | 1440x900 | `f425ca9e4d1e5777618b8363e6dde45bdc9beb6ce17fd343de04b31367065159` |
| `output/playwright/task-00022-settings-plugins-narrow.png` | QA product | 700x840 | `1de1807e84c50fad0fb8d53e63dbbbb6806edd27dd1514e95bac09ab689d025d` |
| `output/playwright/task-00022-settings-skills-wide.png` | QA product | 1440x900 | `ff851018f6c3a80f1017814b65ea850a1727b28daf2ae9b3c3fc3947682a1b7b` |
| `output/playwright/task-00022-settings-skills-narrow.png` | QA product | 700x840 | `fc20e4343abd423fc5ece52560514060001a7303b5ac70a300f219cf47ac73ce` |
| `output/playwright/task-00022-settings-mcp-wide.png` | QA product | 1440x900 | `5541c2e42e0349346f83271366f4d0481c72d272f6e1ee899965e98cc32d3b80` |
| `output/playwright/task-00022-settings-mcp-narrow.png` | QA product | 700x840 | `e83d2e70059b25094fdef8682b6ecf3b73b9ddc80ee7977528615fe34ec7795d` |
| `output/playwright/task-00022-settings-provider-dialog-wide.png` | QA product, shared dialog | 1440x900 | `e0d11e1d6f268ea9491618764d2cb296f62fc810bcd7cfea5010b15d8c0f2a48` |
| `output/playwright/task-00022-settings-provider-dialog-narrow.png` | QA product, shared dialog | 700x840 | `4661749cfd1201fcdc9fb497b25ede7c73e7fcfdbab047f938e8edaaa0e324a2` |
| `output/playwright/task-00022-settings-providers-light.png` | QA product, Light | 1440x900 | `ed31b7869d19bab84611c3a0042b41631582e304e2d1ab1eec5afce04ea1bf87` |
| `output/playwright/task-00022-settings-providers-dark.png` | QA product, Dark | 1440x900 | `5264a705676e1a247603284a1ee428eff95485b1be06b2e90dd5d2c01235cf86` |
| `output/playwright/task-00022-settings-policies-narrow.png` | QA product, supplemental policy rail | 700x800 | `e178f4b09cd0fecff476f90c5a460b686d5bb7237360424ea740c54526abc3b3` |

The current Providers captures contain one route-owned `Providers` title and one support line. The provider list no longer owns a duplicate secondary page title.

## Chat And Composer

| File | Qualification | Dimensions | SHA-256 |
| --- | --- | --- | --- |
| `output/playwright/task-00022-chat-wide.png` | QA product | 1440x900 | `f4ccbbd5a5c6d44811b5370a240f0b2e1c07f8648a5cf7590cf7309689bfbea7` |
| `output/playwright/task-00022-chat-overlay.png` | QA product | 700x840 | `3df739b41c35a8c3131de33a21a62bd963f2a622b9da1466d106127e42af33ef` |
| `output/playwright/task-00022-chat-capability-conflict-wide.png` | QA product | 1440x900 | `492ecc61f7cb0a343939d636853ad884bb9cb1343ea1a61ae6808aeb0c09b6f4` |
| `output/playwright/task-00022-chat-capability-conflict-narrow.png` | QA product | 700x840 | `ba57efc9e6d1fa22aa590039a7e9bb6b803354fde5a2824164449251f54b7984` |
| `output/playwright/task-00022-chat-model-popover-wide.png` | QA product | 1440x900 | `fea72820010c8562dc868d66040c6816c8e35877eaff9470508dd223023b950c` |
| `output/playwright/task-00022-chat-model-browser.png` | QA product | 1440x900 | `5cec4faff45dafbf16d6a39f7c4bdeec036234d211b20d2d9910073cc82aa911` |
| `output/playwright/task-00022-chat-approval-presets-wide.png` | QA product | 1440x900 | `75d6428e72231a25e6087cc0ecda864f5ca776ee48cfec7b7f15ffdaa6829e17` |
| `output/playwright/task-00022-chat-information.png` | QA product | 1440x900 | `9a8df614282d44ae86cde85b1e994cfd4ebce1ae877c50b9044bf7a309a2b166` |
| `output/playwright/task-00022-chat-reply.png` | QA product | 1280x720 | `99891db555444f6927e5e5e24a9721561c5d5853d253fe4fbf5e038d3fb3990b` |
| `output/playwright/task-00022-composer-chat-wide.png` | QA product | 1440x900 | `492ecc61f7cb0a343939d636853ad884bb9cb1343ea1a61ae6808aeb0c09b6f4` |
| `output/playwright/task-00022-composer-files-wide.png` | QA product | 1440x900 | `042d2b100791ca842b4a5c5245cb9ec7fdd0992945ba7b062bf045e223376eca` |
| `output/playwright/task-00022-composer-browser-wide.png` | QA product | 1440x900 | `1d1a38ac8ce5213472ecc07e027dae112fbf5b2f64b0aff6610f5002c90bd2f4` |
| `output/playwright/task-00022-composer-terminal-wide.png` | QA product | 1440x900 | `9e16a91e1e279b22f6d7f601936a5386a027b75cdaa09a81b59d1e8272b43231` |
| `output/playwright/task-00022-composer-mode-popover-wide.png` | QA product | 1440x900 | `2dcbd159f54615fea225466254a9422304493049b0313988514d1fc61a5fb7c8` |

## Artifact Representatives

| File | Qualification | Dimensions | SHA-256 |
| --- | --- | --- | --- |
| `output/playwright/task-00022-file-proposed.png` | QA product | 1280x720 | `15a8cf656b81541397ca21855d0a69b5f56a4ce93fae0dab2f5b46d3376e2479` |
| `output/playwright/task-00022-file-conflict.png` | QA product | 1280x720 | `e6b35925c08981cd1e129f83dcc7ef5af87f0ec12a92c88ae0834f6ba5304b83` |
| `output/playwright/task-00022-file-focused-contextual.png` | QA product | 1280x720 | `4a4200ee70900a53593a65ba009aad70f0d83c14e5b8b2688babb3903ebf73c5` |
| `output/playwright/task-00022-folder-inline.png` | QA product | 1280x800 | `acad44c2a6efd6fa8fbb339c71b7d391081c307bdd118271f892d036ace11c36` |
| `output/playwright/task-00022-folder-focused-overlay.png` | QA product | 390x844 | `7cd016c49ee2767819bf87b9780dfd34387fcfeed8aba0411ef7cd5695d10525` |
| `output/playwright/task-00022-browser-focused-contextual.png` | QA product | 1280x800 | `6d6f71a6537fec1b1477a9951ff878af0904a21ab366be03185829341498dd1b` |
| `output/playwright/task-00022-browser-focused-overlay.png` | QA product | 700x840 | `cdf33cd34c0aac53a13ed74cf4e647edcb364c3f8835f1a596cb00b5a2adfc39` |
| `output/playwright/task-00009-terminal-stdin.png` | QA product, final shared shell | 1440x900 | `2383d3d44fd9b1d889589542e0309b0ec11f425d71b4f3968b4c34c5c52370bc` |
| `output/playwright/task-00009-terminal-stop.png` | QA product, final shared shell | 390x844 | `edd498bd08a4cec0af896f8e0c35b82f9dbff9029035bfffe792d0f8a0de0042` |

These are current shared-shell representatives, not a claim that every provider-specific material state was freshly captured in this corrective pass. The unchanged provider authorities and their broader loading/approval/denial/error/recovery/restart matrices remain evidenced by `.agents/resources/native/task-00008-acceptance.md`, `task-00009-acceptance.md`, and `task-00010-acceptance.md`; the current 47-test Playwright matrix re-exercises their production-composed integrations. The Task 00008–00010 native captures remain the human-reviewable provider-specific evidence for those unchanged authority paths.

## r013 Comparison Inputs

| File | Qualification | Dimensions | SHA-256 |
| --- | --- | --- | --- |
| `output/playwright/task-00022-r013-onboarding-wide.png` | r013 reference | 1440x900 | `ab425e4ef2ae118e70ce4f3b136ee6c64f4a1f29b66a941e76810826e1186ad0` |
| `output/playwright/task-00022-r013-workspace-start-wide.png` | r013 reference | 1440x900 | `fceb02e1411912513d98f4c9f37077e0c3819a407bb98a4b4bc16eb154688b44` |
| `output/playwright/task-00022-r013-chat-wide.png` | r013 reference | 1440x900 | `375d921fde6a461d08d121f0012bbecc257198919f8b1bde9b335dfe6b8e3b48` |
| `output/playwright/task-00022-r013-chat-overlay.png` | r013 reference | 700x800 | `29b113e6fb83f0cb1897bc9e0f7fcc05c2013e9b52b91625ff60dfad8aeb49db` |
| `output/playwright/task-00022-r013-settings-providers-wide.png` | r013 reference | 1440x900 | `b57939c1b80fec5bc540b060c4d479ec3c5341a8f0466caca09c2bf2c4acd1bc` |
