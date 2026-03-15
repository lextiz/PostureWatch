Tagging and Releases

To avoid situations where a tag is pushed that references an older workflow or untested commit, please use the "Preflight Tagger" GitHub Actions workflow instead of creating tags locally and pushing them directly.

How to create a release (recommended):

1. Open the repository on GitHub -> Actions -> "Preflight Tagger" -> Run workflow.
2. Enter the branch you want to tag (e.g., main) and the tag name (e.g., v0.1.2-fix-camera-...).
3. The workflow will build the checked-out branch and, on success, create the annotated tag and release.

If you prefer CLI, use the included ci-self-improve/tagger-dispatch.sh script with a GITHUB_TOKEN that has repo permissions.

Rationale: this guarantees the branch is built with the same CI environment and dependencies before a tag is created, preventing "old-workflow" surprises.
