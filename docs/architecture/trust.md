# Trust

## Authority

- client
- server

## Data

- diff
- state

## Variant

- trusted
- untrusted

## Lifecycle

1. client
   - human issues action `diff[untrusted{client}]`:
     - compute `state[untrusted{client}]` = `execute(state[untrusted{client}], diff[untrusted{client}])`
     - transmit `state[untrusted{client}]` and `diff[untrusted{client}]`
1. server
   - receives `state[untrusted{client}]` and `diff[untrusted{client}]`
     - compute `validate(state[trusted{server}], diff[untrusted{client}])`
       - allowed:
         - `diff[trusted{server}]` = `diff[untrusted{client}]`
       - denied:
         - transmit: `state[trusted{server}]`
         - end...
     - compute `state[trusted{server}]` = `execute(state[trusted{server}], diff[trusted{server}])`
     - compute `drift(state[trusted{server}], state[untrusted{client}])`
       - close:
         - `state[trusted{server}]` = `state[untrusted{client}]`
       - far:
         - transmit: `state[trusted{server}]`
     - broadcast `state[trusted{server}]` and `diff[trusted{server}]`
1. server(peer)
   - receives `state[trusted{server}]` and `diff[trusted{server}]`
     - apply `state[trusted{server}]`
     - transmit `state[trusted{server}]` and `diff[trusted{server}]`
1. client(peer)
   - receives `state[trusted{server}]` and `diff[trusted{server}]`
     - compute `state[trusted{client}]` = `execute(state[trusted{client}], diff[trusted{server}])`
     - compute `drift(state[trusted{server}], state[trusted{client}])`
       - close:
         - do nothing...
       - far:
         - `state[trusted{client}] = state[trusted{server}]`
