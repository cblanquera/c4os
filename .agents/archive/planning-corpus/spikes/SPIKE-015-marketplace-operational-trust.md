# Objective

Determine whether marketplace support should exist before plugin trust, signing, update verification, and support operations are defined.

# Context

Marketplace specs include local and team marketplace manifests, but reviews warn that marketplace UX implies trust, curation, liability, vulnerability response, and support obligations.

# Questions To Answer

 - Is marketplace support needed for MVP validation?
 - Is local plugin install enough for early phases?
 - Are unsigned remote marketplace entries allowed?
 - How are plugin updates verified?
 - What is the vulnerability advisory and removal process?
 - Who is responsible for third-party plugin damage?
 - Do team marketplaces make sense without team policy administration?

# Hypothesis

Marketplace support should be deferred until plugin execution trust and source integrity are solved.

# Investigation Plan

 - Review marketplace spec and plugin trust ADRs.
 - List operational responsibilities implied by marketplace support.
 - Compare local-only install, local manifest, team manifest, and hosted marketplace models.
 - Identify minimum trust requirements before marketplace exposure.
 - Document deferral recommendation if operational burden exceeds MVP value.

# Success Criteria

 - Marketplace phase recommendation is documented.
 - Operational trust requirements are listed.
 - Update and source integrity questions are answered at research level.
 - Marketplace MVP inclusion is accepted or rejected.

# Decisions Unlocked

 - ADR-015: Marketplace Model.
 - ADR-016: Plugin Trust And Execution.
 - ADR-014: Plugin System Compatibility.

# Estimated Effort

1 to 3 product/security days.

