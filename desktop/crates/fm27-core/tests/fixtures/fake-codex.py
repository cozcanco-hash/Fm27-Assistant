"""Deterministic protocol fixture; no network or account access."""
import json
import sys

mode = sys.argv[1]


def send(value):
    print(json.dumps(value, ensure_ascii=False), flush=True)


for line in sys.stdin:
    message = json.loads(line)
    method = message.get("method")
    params = message.get("params", {})
    if method == "initialize":
        send({"id": message["id"], "result": {"userAgent": "fixture"}})
    elif method == "account/read":
        send({"id": message["id"], "result": {"account": {"type": "chatgpt", "planType": "pro"}}})
    elif method == "thread/start":
        assert params["sandbox"] == "read-only"
        assert params["approvalPolicy"] == "never"
        assert params["ephemeral"] is True
        send({"id": message["id"], "result": {"thread": {"id": "thread-1"}}})
    elif method == "turn/start":
        assert params["sandboxPolicy"]["type"] == "readOnly"
        assert params["sandboxPolicy"]["access"]["type"] == "restricted"
        assert "Demo" in params["input"][0]["text"]
        send({"id": message["id"], "result": {"turn": {"id": "turn-1", "status": "inProgress"}}})
        if mode == "failed":
            send({"method": "turn/completed", "params": {"turn": {"id": "turn-1", "status": "failed"}}})
            continue
        send({"method": "item/agentMessage/delta", "params": {"itemId": "z", "delta": "Bir"}})
        if mode == "cancel":
            continue
        send({"method": "item/agentMessage/delta", "params": {"itemId": "z", "delta": "inci"}})
        send({"method": "item/completed", "params": {"item": {"id": "z", "type": "agentMessage", "text": "Birinci"}}})
        send({"method": "item/completed", "params": {"item": {"id": "a", "type": "agentMessage", "text": "İkinci"}}})
        send({"method": "turn/completed", "params": {"turn": {"id": "turn-1", "status": "completed"}}})
    elif method == "turn/interrupt":
        send({"id": message["id"], "result": {}})
        send({"method": "turn/completed", "params": {"turn": {"id": "turn-1", "status": "interrupted"}}})
