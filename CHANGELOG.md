# Changelog

All notable changes to this crate are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the crate
follows [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- `Client` and `ClientBuilder` (token, base URL, timeouts, user agent).
- Shippings: create, get, list with pagination, cancel, tracking, proof of
  delivery, delivery PIN, delivery instructions, callback history, labels.
- Estimates: create and confirm.
- Coverage: check and working zones; fleet schedules.
- Webhook configuration: get, set, delete.
- `webhook` module to parse incoming callbacks and verify their
  `Authorization` key in constant time.
- Typed errors with PedidosYa error codes.
- Live test suite (`tests/live.rs`, ignored by default) using test shippings.

### Verified against the live API (differences from the OpenAPI spec)

- Callback history uses `callbackUrl` (not `callBackUrl`), wraps the URL in
  brackets, and reports `responseCode: -1` when the endpoint is unreachable.
- Shipping list accepts at most 14 days between `fromDate` and `toDate`;
  validated locally (`ListShippings::MAX_RANGE_DAYS`).
- Invalid token errors are `400 INVALID_PARAMS` with a `messages` list;
  `ApiError::is_unauthorized` and `message` handle it.
- Undocumented fields kept: `Item::category_name`, `Item::category_id`,
  `Waypoint::last_updated_instructions`.
- Undocumented error codes: `DROP_OFF_OUT_OF_ZONE`, `MAX_ITEMS_EXCEEDED`,
  `INVALID_PARAMS`, `INVALID_TOKEN`.
- `PUT /webhooks-configuration` upserts per topic and environment; other
  subscriptions are kept. Documented on `Webhooks::set`.
- Fleet schedules can cross midnight UTC (`Schedule::crosses_midnight`,
  `Schedule::contains`).
