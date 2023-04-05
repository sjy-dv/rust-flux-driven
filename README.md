## Explain
This is the most basic connection between influxdb and rust.
Rust is a language that can process large amounts of data, and InfluxDB is also a database that can collect large amounts of logs.

So the combination of the two looked good, so I wrote the code.


## What you should pay attention to

``` rust
let thread_http_server = HttpServer::new(move || {

let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.spawn(async move {

thread_http_server.run().await?;
```
As you can see, the above code starts by creating separate actix and InfluxDB threads.

The reason I wrote it as above is that when I tie a thread like this, **actix doesn't start it first, so I get a task creation error.**

You will need to do additional separate threading to collect additional events.

I implemented both producer and consumer in the Rust server with the channel concept provided by tokio, but depending on your architecture, it can be transformed into a consumer code that only consumes.