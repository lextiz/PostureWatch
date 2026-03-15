#!/usr/bin/env bash
# Dispatch the Preflight Tagger workflow on GitHub Actions.
# Usage: ./tagger-dispatch.sh <branch> <tag>
set -e
if [ "$#" -ne 2 ]; then
  echo "Usage: $0 <branch> <tag>"
  exit 2
fi
BRANCH=$1
TAG=$2
REPO_OWNER="lextiz"
REPO_NAME="PostureWatch"
if [ -z "$GITHUB_TOKEN" ]; then
  echo "Set GITHUB_TOKEN env var with repo write permissions"
  exit 2
fi
curl -s -X POST -H "Authorization: token $GITHUB_TOKEN" -H "Accept: application/vnd.github+json" \
  https://api.github.com/repos/$REPO_OWNER/$REPO_NAME/actions/workflows/tagger.yml/dispatches \
  -d "{\"ref\": \"$BRANCH\", \"inputs\": {\"branch\": \"$BRANCH\", \"tag\": \"$TAG\"}}"

echo "Dispatched tagger for $BRANCH -> $TAG"
