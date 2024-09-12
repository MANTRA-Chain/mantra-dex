# Fee Collector

The Fee Collector is a contract that collects the fees accrued by the protocol. Whenever a pool or a farm is created,
as well as swaps, a fee is sent to the Fee Collector. As of now, the Fee Collector does not have any other function.

```mermaid
---
title: Fee Collection Mechanism
---
graph LR
    A[Pool Manager] --> B[Create Pool]
    A --> D[Perform Swap]

    H[Farm Manager] --> I[Create farm]
    H --> L[User emergency withdraw]

    B --> J{Fee Collected}
    D --> J
    L --> J
    I --> J
    J --> K[Fee Collector]
```
