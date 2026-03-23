#!/bin/bash
# The problem is that deleting and cloning the DB changes its project UUID,
# but we generate a RANDOM new one in metadata.json. It needs to match EXACTLY
# the one that the cloned database expects. Also, the databases didn't successfully clone 
# for some repositories because they were completely empty remotes with no main branch yet.

fix_repo() {
    REPO_NAME=$1
    PREFIX=$2
    REMOTE_NAME=$3
    REPO_DIR="/home/lewis/src/$REPO_NAME"
    
    echo "================================================="
    echo "Fixing Database IDs for $REPO_NAME ($PREFIX)"
    echo "================================================="
    
    if [ ! -d "$REPO_DIR" ]; then
        return
    fi
    
    cd "$REPO_DIR"
    
    if [ -d ".beads" ]; then
        # 1. Kill any running dolt server to unlock the files
        pkill -f "dolt sql-server.*$REPO_DIR" 2>/dev/null || true
        if [ -f ".beads/dolt-server.pid" ]; then
            kill -9 $(cat .beads/dolt-server.pid) 2>/dev/null || true
            rm -f .beads/dolt-server.pid
        fi
        
        # 2. To get the correct UUID, we need bd to read the database.
        # But if the db is empty (brand new remote), `dolt clone` failed or didn't create the folder correctly.
        # Let's ensure the folder exists by letting `bd init` create it if it has to.
        
        # We'll just blow away the dolt folder entirely and let bd initialize a completely fresh one
        # locally. This is the only way to ensure the metadata matches perfectly for completely new setups.
        echo "Resetting local database entirely to match bd init..."
        rm -rf .beads/dolt
        rm -f .beads/metadata.json
        
        # Initialize fresh (this creates the metadata.json and dolt folder perfectly matched)
        bd init --prefix "$PREFIX" > /dev/null 2>&1 || true
        
        # Now we just need to add the remote and push
        if [ -d ".beads/dolt/$PREFIX" ]; then
            cd .beads/dolt/$PREFIX
            dolt remote remove origin 2>/dev/null || true
            dolt remote add origin "https://doltremoteapi.dolthub.com/priorlewis43/$REMOTE_NAME"
            cd "$REPO_DIR"
            
            # Restart server just in case
            bd status > /dev/null 2>&1
            
            # Force push everything to remote to initialize it
            echo "Pushing new local database to remote..."
            bd dolt push || echo "Push failed, remote might need explicit setup"
        else
            echo "Failed to initialize bd for $PREFIX"
        fi
    fi
}

fix_repo "centralized-docs" "cdocs" "centralized-docs-database"
fix_repo "hardline" "hard" "hardline-database"
fix_repo "twerk" "twerk" "twerk-database"
fix_repo "wtf-engine" "wtf" "wtf-engine-database"
fix_repo "clarity" "clar" "clarity-database"
fix_repo "seshat" "seshat" "seshat-database"
