#!/bin/bash
SESSION="poly"

tmux new-session -d -s $SESSION

# Pane 0: top-left
tmux send-keys -t $SESSION 'make start-service-api' C-m

# Split vertical (bottom)
tmux split-window -v -t $SESSION
tmux send-keys -t $SESSION.1 'make start-grpc-server' C-m

# Select top pane and split horizontally (top-right)
tmux select-pane -t $SESSION.0
tmux split-window -h -t $SESSION
tmux send-keys -t $SESSION.2 'make start-websocket-server' C-m

# Attach
tmux attach-session -t $SESSION