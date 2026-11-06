#!/usr/bin/env python3
"""Protocol-only test peer. Never contacts an AI service."""
import json
import pathlib
import sys
import threading

lock = threading.Lock()
turn = 0
active = {}


def emit(value):
    with lock:
        print(json.dumps(value, ensure_ascii=False), flush=True)


def finish(turn_id, text):
    if active.pop(turn_id, None) is None:
        return
    emit({'method': 'item/completed', 'params': {'item': {'id': turn_id, 'type': 'agentMessage', 'phase': 'final_answer', 'text': text}}})
    emit({'method': 'turn/completed', 'params': {'turn': {'id': turn_id, 'status': 'completed'}}})


for line in sys.stdin:
    message = json.loads(line)
    method, params = message.get('method'), message.get('params', {})
    result = {}
    if method == 'account/read':
        result = {'account': {'type': 'chatgpt', 'planType': 'protocol-test'}}
    elif method == 'model/list':
        result = {'data': [{'model': 'protocol-test', 'displayName': 'Protocol test only', 'isDefault': True}]}
    elif method == 'thread/start':
        result = {'thread': {'id': 'test-thread'}}
    elif method == 'turn/start':
        turn += 1
        turn_id = f'test-turn-{turn}'
        text = params['input'][0]['text']
        pathlib.Path('captured-context.txt').write_text(text)
        answer = 'TEST: shared conversation received' if 'TEST:' in text else 'TEST: first answer'
        emit({'id': message['id'], 'result': {'turn': {'id': turn_id}}})
        emit({'method': 'item/agentMessage/delta', 'params': {'itemId': turn_id, 'delta': 'TEST: working'}})
        active[turn_id] = True
        timer = threading.Timer(0.6, finish, args=(turn_id, answer))
        timer.daemon = True
        timer.start()
        continue
    elif method == 'turn/interrupt':
        turn_id = params['turnId']
        active.pop(turn_id, None)
        emit({'id': message['id'], 'result': {}})
        emit({'method': 'turn/completed', 'params': {'turn': {'id': turn_id, 'status': 'interrupted'}}})
        continue
    if 'id' in message:
        emit({'id': message['id'], 'result': result})
