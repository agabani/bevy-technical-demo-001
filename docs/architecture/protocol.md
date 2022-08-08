# Protocol

## V1

### Keep Alive

1. self: `Ping`
1. peer: `Pong`

### Spawn

Client initiated:

1. client: `Spawn`
1. server: `Spawned` { id, x, y }

### Despawn

Client initiated:

1. client: `Despawn`
1. server: `Despawned` { id }

Server initiated:

1. server: `Despawned` { id }
