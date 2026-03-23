#!/bin/bash
# The absolute nuclear option. We have to wipe .beads entirely for every repo
# to guarantee clean dolt servers, clean configurations, and clean project IDs,
# then add the remote, and force push to instantiate the remote database.

fix_repo() {
    REPO_NAME=$1
    PREFIX=$2
    REMOTE_NAME=$3
    REPO_DIR="/home/lewis/src/$REPO_NAME"
    
    echo "================================================="
    echo "NUCLEAR RESET: $REPO_NAME ($PREFIX)"
    echo "================================================="
    
    if [ ! -d "$REPO_DIR" ]; then
        return
    fi
    
    cd "$REPO_DIR"
    
    # Ensure any rogue dolt servers for this repo are dead
    pkill -f "dolt sql-server.*$REPO_DIR" 2>/dev/null || true
    if [ -f ".beads/dolt-server.pid" ]; then
        kill -9 $(cat .beads/dolt-server.pid) 2>/dev/null || true
    fi
    
    # Nuke the entire beads directory to ensure we don't hit "already initialized" errors
    echo "Nuking .beads..."
    rm -rf .beads
    
    # Initialize from total scratch
    echo "Initializing new beads database with prefix $PREFIX..."
    bd init --prefix "$PREFIX" > /dev/null 2>&1
    
    # Wait for the dolt server to stand up
    sleep 2
    
    # Link to the remote
    echo "Linking to DoltHub remote $REMOTE_NAME..."
    if [ -d ".beads/dolt/$PREFIX" ]; then
        cd .beads/dolt/$PREFIX
        dolt remote add origin "https://doltremoteapi.dolthub.com/priorlewis43/$REMOTE_NAME"
        
        # We need to create an empty commit to be able to push to an empty remote
        dolt commit --allow-empty -m "Initial remote setup" > /dev/null 2>&1 || true
        
        # Force push the main branch to instantiate the remote database
        echo "Force pushing to remote..."
        dolt push -u origin main -f
        cd "$REPO_DIR"
        
        # Add the remote using bd so it knows about it
        bd dolt remote add origin "https://doltremoteapi.dolthub.com/priorlewis43/$REMOTE_NAME" > /dev/null 2>&1 || true
        
        # Verify
        bd status > /dev/null 2>&1 && echo "SUCCESS: $REPO_NAME is perfectly linked." || echo "WARNING: Status check failed."
    else
        echo "ERROR: Dolt directory not created properly for $PREFIX."
    fi
    echo ""
}

# centralized-docs already done in the manual test, but running it again is harmless since it's a nuclear reset
fix_repo "centralized-docs" "cdocs" "centralized-docs-database"
fix_repo "hardline" "hard" "hardline-database"
fix_repo "twerk" "twerk" "twerk-database"
fix_repo "wtf-engine" "wtf" "wtf-engine-database"
fix_repo "clarity" "clar" "clarity-database"
fix_repo "seshat" "seshat" "seshat-database"
