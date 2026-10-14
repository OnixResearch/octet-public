# System comprehension guide

Tiger Style system notes should teach readers in this order: structure first, then function, then selected depth. Start with map that helps reader place moving parts, then walk one real flow, then zoom in on weak spots that need more care.

## Reader, boundary, and system slice

Name intended reader before anything else. Say whether note is for new contributor, reviewer, operator, or maintainer. Draw boundary around one system slice so review stays concrete instead of trying to explain whole world at once.

- Reviewer prompt: Who is supposed to understand this note after one pass?
- Reviewer prompt: What boundary and system slice are in scope, and what is deliberately out of scope?
- Checklist: State audience, boundary, and slice in first screen of document.

## Structure before function

Readers need static map before dynamic story. Show major components, ownership boundaries, interfaces, and dependency edges before explaining request path, state transition, or recovery behavior.

- Reviewer prompt: Can reader point to major components and boundaries before reading control flow?
- Reviewer prompt: Which dependency edges or ownership seams matter most for this slice?
- Checklist: Explain components and boundaries before any end-to-end walkthrough.

## Functional walkthrough after structure

Once map is clear, walk one primary end-to-end flow. Use concrete entry point, major hops, and exit condition. Keep walkthrough anchored to components already named.

- Reviewer prompt: What one primary flow best explains how this slice behaves?
- Reviewer prompt: Does each step in walkthrough refer back to named components instead of introducing mystery boxes?
- Checklist: Prefer one complete walkthrough over many half-explained paths.

## Deep dives, weak spots, and ownership

After reader has map and primary flow, add selected depth where confusion or risk is highest. Call out weak spots, ownership gaps, fragile assumptions, and follow-up areas. Do not bury them under generic prose.

- Reviewer prompt: Which part of this slice still needs deeper explanation because risk or confusion is concentrated there?
- Reviewer prompt: What weak spot or ownership gap should next maintainer know before making changes?
- Checklist: Deep dives should stay selective and should name owner or follow-up path for unclear areas.

## Top-level docs, crate roots, and dependency maps

System comprehension starts before source dive. README, crate roots, module trees, and dependency maps should help reader orient quickly. If those surfaces mislead reader, later deep docs will not rescue first impression.

- Reviewer prompt: Does README point reader to right entry point for this slice?
- Reviewer prompt: Do crate roots, module exports, and dependency maps reveal structure instead of hiding it?
- Checklist: Review top-level docs and crate-root surfaces as part of architecture legibility, not as separate polish pass.

## Reviewer prompts for 2-minute legibility

Ask whether reviewer can sketch major slice on whiteboard after two minutes with document. If answer is no, explanation still starts too deep, skips boundaries, or spreads crucial facts across too many places.

- Reviewer prompt: After two minutes, can reviewer sketch major components, boundaries, and one primary flow from memory?
- Reviewer prompt: Which missing label, boundary, or ownership note blocks that quick sketch?
- Checklist: Send work back when reader cannot draw subsystem map quickly without oral backup.
