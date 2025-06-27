#!/bin/bash

set -e

SESSION="qitech"

# Start a new tmux session, detached
tmux new-session -d -s $SESSION

# Split first window vertically (50%)
tmux split-window -h -t $SESSION

# Send commands to both panes (adjust commands as needed)
tmux send-keys -t $SESSION:0.0 'cd server && cargo watch -x run --features mock-machine ' C-m
tmux send-keys -t $SESSION:0.1 'cd electron && npm start' C-m

# Create a new window
tmux new-window -t $SESSION -n 'editors'

# Split new window vertically
tmux split-window -h -t $SESSION:1

# Send commands to both panes
tmux send-keys -t $SESSION:1.0 'cd server/src && $EDITOR .' C-m
tmux send-keys -t $SESSION:1.1 'cd electron/src && $EDITOR .' C-m

# Attach to the session
exec tmux attach -t $SESSION
