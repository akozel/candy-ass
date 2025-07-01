## Motivation:

---

This application was developed as part of a research project to evaluate 
the `expressiveness` and `performance` of the `Rust` programming language.
The selected domain is data preparation for `backtesting` trading strategies,
which is a component of my product previously implemented in Java.

### Application structure
The workspace of this application consists of two crates: `core` and `backtest`.

The backtest crate, in particular, is divided into two applications:
- `History Downloader` - fetch historical data into local storage
- `History Reproducer` - replay historical data from local storage

Both applications are optimized to achieve blazing fast performance.

## History Downloader

---

*History Downloader* application connects to the exchange (Binance) and performs requests 
using **multiple threads**. It respects exchange rate limits and **ensures backpressure**
is properly handled. The retrieved data is accumulated in a buffer (50 items by default),
then sliced into batches and inserted into the database in groups (bulk insert).



![downloader.png](./downloader.png)


## History Reproducer

---

This application loads data from the database in slices and pushes it into the stream buffer. 
By default, it performs prefetching for 4 slices. As a CPU-level optimization,
the conversion of database objects into application objects is performed in 
a synchronous fork-join pool running outside the `single threaded` `event loop`. 
Additionally, data compression in the database client is disabled, since the database is assumed 
to be deployed in a local network and the main bottleneck is the CPU calculations
of the back-test application.

Once you fetch stream, it is recommended to accumulate it into `ring buffer`
data structures for further processing.

![reproducer.png](./reproducer.png)


