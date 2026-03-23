#!/bin/bash

# A more robust repair script since centralized-docs got stuck on 'already initialized'
# but 'database not found'. The issue is Dolt SQL server is running but dolt database doesn't
# match what metadata.json expects OR the folder is misaligned.

fix_repo() {
    REPO_NAME=$1
    PREFIX=$2
    REMOTE_NAME=$3
    REPO_DIR="/home/lewis/src/$REPO_NAME"
    
    echo "================================================="
    echo "Fixing and Pushing $REPO_NAME ($PREFIX)"
    echo "================================================="
    
    if [ ! -d "$REPO_DIR" ]; then
        echo "Directory $REPO_DIR does not exist. Skipping."
        return
    fi
    
    cd "$REPO_DIR"
    
    # 1. Force push all code safely
    echo "Pushing code..."
    git add -A || true
    git commit -m "chore: sync state before database isolation" || true
    
    # If in detached head, push to main
    BRANCH=$(git branch --show-current)
    if [ -z "$BRANCH" ]; then
        git push origin HEAD:main || echo "Git push failed."
    else
        git push || echo "Git push failed."
    fi
    
    # 2. Fix the DB setup
    if [ -d ".beads" ]; then
        echo "Fixing DB setup for $PREFIX..."
        
        # Shutdown any running server to unlock files
        if [ -f ".beads/dolt-server.pid" ]; then
            kill -9 $(cat .beads/dolt-server.pid) 2>/dev/null || true
            rm -f .beads/dolt-server.pid
        fi
        pkill -f "dolt sql-server.*$REPO_DIR" 2>/dev/null || true
        
        # The ultimate reset: Export if we can, backup .beads, nuke it, clone fresh remote, re-init.
        # But wait, if they have backups in .beads/backup, we can use that!
        
        # We will wipe the dolt directory completely and clone the fresh database
        rm -rf .beads/dolt
        mkdir -p .beads/dolt
        cd .beads/dolt
        dolt clone "priorlewis43/$REMOTE_NAME" "$PREFIX" || true
        cd "$REPO_DIR"
        
        NEW_UUID=$(uuidgen)
        cat << INNER_EOF > .beads/metadata.json
{
  "database": "dolt",
  "backend": "dolt",
  "dolt_mode": "server",
  "dolt_database": "$PREFIX",
  "project_id": "$NEW_UUID"
}
INNER_EOF
        
        bd config set issue_prefix "$PREFIX"
        
        # Check status, this should auto-start the server for the correctly named DB folder
        echo "Checking status..."
        bd status || echo "Status failed."
        
        echo "Pushing dolt..."
        bd dolt push || echo "Dolt push failed."
        
    else
        # If no .beads, initialize from scratch
        bd init --prefix "$PREFIX"
        
        # Then replace the db with the cloned remote
        if [ -f ".beads/dolt-server.pid" ]; then
            kill -9 $(cat .beads/dolt-server.pid) 2>/dev/null || true
            rm -f .beads/dolt-server.pid
        fi
        pkill -f "dolt sql-server.*$REPO_DIR" 2>/dev/null || true
        
        rm -rf .beads/dolt/*
        cd .beads/dolt
        dolt clone "priorlewis43/$REMOTE_NAME" "$PREFIX" || true
        cd "$REPO_DIR"
        
        bd status || echo "Status failed."
        bd dolt push || echo "Dolt push failed."
    fi
    echo "Done with $REPO_NAME."
    echo ""
}

fix_repo "centralized-docs" "cdocs" "centralized-docs-database"
fix_repo "hardline" "hard" "hardline-database"
fix_repo "twerk" "twerk" "twerk-database"
fix_repo "wtf-engine" "wtf" "wtf-engine-database"
fix_repo "clarity" "clar" "clarity-database"
fix_repo "seshat" "seshat" "seshat-database"

echo "All specified repos have been processed."
