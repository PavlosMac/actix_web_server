# Actix Web Crawler

- Exposing two endpoints, one to run a heavy IO task for domain link scraping, one to return results as list.
- Uses in-memory HashMap to persist global state.
- Should return HttpResponse or HttpResponseError.

## Useage:

Start server:

```sh
> RUST_LOG=info cargo run
```

Send POST request with domain target:

```sh
> curl -i --request POST \
--url http://localhost:8080/process \
--header 'content-type: application/json' \
--data '{
        "domain": "github.com"
}'
```

GET request for results retrieval by target:

```sh
> curl -i --request GET http://localhost:8080/results\?domain=github.com'
```
