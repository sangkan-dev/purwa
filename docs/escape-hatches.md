# Escape hatches

Purwa favors conventions but exposes the underlying stack. You are not locked into a black box.

## Axum `Router` and `Router<()>`

**`router_from_inventory()`** returns **`Router<()>`** (unit state). That matches inventory-registered handlers that do not use Axum **`State`**.

If your handlers need **`State<AppState>`** (e.g. **`PgPool`**, config), you typically:

- Build a **`Router<AppState>`** (or a state type you own), and
- **Merge** or **nest** routes from the inventory router **after** applying **`with_state`**, or restructure so shared data uses **`Extension`** / **`FromRef`** slices as documented in **`AppState`** ([`purwa-core`](../purwa-core/src/lib.rs)).

Exact composition is **application-specific**; Purwa does not yet force one macro for “inventory + typed state” in all cases. See the routing note in the [README](../README.md).

## Raw Axum and Tower

- **`purwa::axum`** is re-exported (hidden in docs) for drop-down to Axum types.
- You can add routes, layers, and services using normal Axum/Tower APIs on the same **`Router`**.

## SQLx and `PgPool`

- **`AppState`** holds an optional **`PgPool`** ([`purwa-core`](../purwa-core/src/lib.rs)).
- Use **`sqlx::query_as`**, **`query!`**, and migrations through **`purwa_orm::connect_pool`**, **`migrate_up`**, etc., or your own SQLx setup.
- There is **no** official lightweight mock for **`PgPool`**; tests either avoid the pool (routing only) or use a real disposable database ([README](../README.md), **`purwa-testing`**).

## SeaORM

Enable the **`sea-orm`** feature on **`purwa`** when you need the SeaORM bridge from **`purwa-orm`**. SQLx remains the default path.

## Inertia

Use **`purwa-inertia`** types directly (`InertiaRequest`, `InertiaRenderContext`, …). You can customize the HTML shell injection and error mapping (**`respond_purwa_error`**) without forking the crate.

## Session and auth

**`purwa-auth`** builds on **`tower-sessions`** and **`axum-login`**. Advanced setups can replace pieces if you stay compatible with Axum’s layer model (see crate docs and tests).

## Generators

**`empu make:*`** writes starter files into your tree. They are **yours** to edit; deleting or replacing them does not break the framework—only your app’s compile.
