#!/bin/bash
# Push the source code first so we don't get detached head / uncommitted errors

push_code() {
    REPO_NAME=$1
    REPO_DIR="/home/lewis/src/$REPO_NAME"
    
    echo "================================================="
    echo "Pushing code: $REPO_NAME"
    echo "================================================="
    
    if [ ! -d "$REPO_DIR" ]; then
        return
    fi
    
    cd "$REPO_DIR"
    
    # Check if inside git
    if git rev-parse --is-inside-work-tree > /dev/null 2>&1; then
        # Add everything, commit and push
        git add -A || true
        git commit -m "chore: sync state before database isolation" || true
        
        # If in detached HEAD, switch to main and merge, or just push HEAD to main
        BRANCH=$(git branch --show-current)
        if [ -z "$BRANCH" ]; then
            git push origin HEAD:main || true
        else
            git push origin "$BRANCH" || true
        fi
    fi
}

push_code "centralized-docs"
push_code "hardline"
push_code "twerk"
push_code "wtf-engine"
push_code "clarity"
push_code "seshat"
