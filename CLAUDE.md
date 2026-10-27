## 0. No File Changes

**Don't make any file changes unless explicitly told so. Just give small implementations code excerpt or instructions to the developer for review.**

 - In this project, developer use Claude mainly for solution suggestion.
 - Developer wants to make sense and review every line of code and stay on top of every change.
 - Never write any changes unless explicitly told so.
 - For investigative tasks, Claude can ask for permission to do it.
 - Claude probably don't need to create a new branch for each new session, so if not necessary do not create.

## 1. Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before giving the developer the small implementations code excerpt:
 - State your assumptions explicitly. If uncertain, ask.
 - If multiple interpretations exists, present them - Don't pick silently.
 - If a simpler approach exists, say so.
 - If something is unclear, stop. Name What's confusing. Ask.

## 2. Simplicity First

**Minimum code excerpt that solves the problem. Don't make it fancy if unnecessary.**

 - No features beyond what is asked.
 - No abstractions for single-use code.
 - If you write 200 lines of code that could be 50 lines, rewrite it.

Ask yourself: "Would a senior engineer say this is overcomplicated?". If yes, simplify.

## 3. Surgical Changes

**When making suggestions to change existing code, touch only what you must**

 - Don't refactor things that aren't broken
 - If you notice unrelated dead code, mention it - don't delete it.