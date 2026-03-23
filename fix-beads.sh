#!/bin/bash
set -e

fix_repo() {
    REPO_NAME=$1
    PREFIX=$2
    REMOTE_NAME=$3
    REPO_DIR="/home/lewis/src/$REPO_NAME"
    
    echo "================================================="
    echo "Fixing $REPO_NAME..."
    echo "================================================="
    
    if [ ! -d "$REPO_DIR" ]; then
        echo "Directory $REPO_DIR does not exist. Skipping."
        return
    fi
    
    cd "$REPO_DIR"
    
    # Check if beads exists
    if [ ! -d ".beads" ]; then
        echo "No .beads directory found. Initializing fresh..."
        bd init --prefix "$PREFIX"
    else
        echo "Found existing .beads directory."
        
        # Kill any running dolt server for this repo specifically
        if [ -f ".beads/dolt-server.pid" ]; then
            PID=$(cat .beads/dolt-server.pid)
            echo "Killing dolt server (PID $PID)..."
            kill -9 $PID 2>/dev/null || true
            rm -f .beads/dolt-server.pid
        fi
    fi
    
    # Wait a second to ensure port is freed
    sleep 1
    
    # 1. Update metadata.json to ensure correct database mapping
    echo "Fixing metadata.json..."
    if [ -f ".beads/metadata.json" ]; then
        # Use a temporary file to safely update metadata.json without jq
        NEW_UUID=$(uuidgen)
        cat << INNER_EOF > .beads/metadata.tmp.json
{
  "database": "dolt",
  "backend": "dolt",
  "dolt_mode": "server",
  "dolt_database": "$PREFIX",
  "project_id": "$NEW_UUID"
}
INNER_EOF
        mv .beads/metadata.tmp.json .beads/metadata.json
    fi
    
    # 2. Fix the prefix in bd config
    echo "Setting issue prefix to $PREFIX..."
    bd config set issue_prefix "$PREFIX"
    
    # 3. Handle the Dolt backend
    echo "Fixing Dolt remote and database structure..."
    if [ -d ".beads/dolt/seshat" ] && [ "$PREFIX" != "seshat" ]; then
        echo "Found bleeding 'seshat' database. Renaming..."
        mv .beads/dolt/seshat ".beads/dolt/$PREFIX"
    fi
    
    # If there's a dolt db but it's empty or corrupt, or we just renamed it
    if [ -d ".beads/dolt/$PREFIX" ]; then
        cd ".beads/dolt/$PREFIX"
        # Reset remote
        dolt remote remove origin 2>/dev/null || true
        dolt remote add origin "https://doltremoteapi.dolthub.com/priorlewis43/$REMOTE_NAME"
        
        # We try to pull, but if it's a new remote with no common ancestor, it might fail.
        # So we just ignore errors here, bd dolt push will handle the rest.
        dolt pull origin main 2>/dev/null || true
        cd "$REPO_DIR"
    else
        echo "No dolt db found for $PREFIX. We will let bd recreate it."
    fi
    
    # 4. Verify status and push
    echo "Checking status and pushing..."
    bd status
    echo "Pushing to remote..."
    bd dolt push || echo "Push failed, but environment is isolated."
    
    echo "Done with $REPO_NAME."
    echo ""
}

fix_repo "centralized-docs" "cdocs" "centralized-docs-database"
fix_repo "hardline" "hard" "hardline-database"
fix_repo "twerk" "twerk" "twerk-database"
fix_repo "wtf-engine" "wtf" "wtf-engine-database"
fix_repo "clarity" "clar" "clarity-database"

echo "All specified repos have been processed."
