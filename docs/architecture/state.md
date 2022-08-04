# State

## Initialization

Inventory:

1. server c: join inventory (0)
1. server c: discover inventory (0)
   - if alone:
     - load inventory (0)
   - if not alone:
     - Propagation - Server <--> Server

Location:

1. server c: join location (x0,y0)
1. server c: discover location (x0,y0)
   - if alone:
     - load location (x0,y0)
   - if not alone:
     - Propagation - Server <--> Server

## Propagation - Server <--> Client

Inventory:

1. client: subscribe to inventory (0)
1. server: `inventory(0)`
   - ...data...

Location:

1. client: what is my current location?
1. server: `(x0,y0)`
1. client: subscribe to location (x0,y0)
1. server: `location(x0,y0)`
   - ...data...

## Propagation - Server <--> Database

Inventory:

1. server: `inventory(0)`
   - ...data...

Location:

1. server: `location(x0,y0)`
   - ...data...

## Propagation - Server <--> Server

Location:

1. server b: subscribe to location (x0,y0)
1. server a: `location(x0,y0)`
   - ...data...
1. server a: subscribe to location (x0,y0)
1. server b: `location(x0,y0)`
   - ...data...
1. server c: subscribe to location (x0,y0)
1. server a: `location(x0,y0)`
   - ...data...
1. server c: subscribe to location (x0,y0)
1. server b: `location(x0,y0)`
   - ...data...
1. server a: subscribe to location (x0,y0)
1. server c: `location(x0,y0)`
   - ...data...
1. server b: subscribe to location (x0,y0)
1. server c: `location(x0,y0)`
   - ...data...
