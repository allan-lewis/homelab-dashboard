# Homelab Dashboard

## Development

Open three terminal windows/panes:

### Editor

```bash
nvim .
```

### UI Build

Rebuild the frontend after making UI changes:

```bash
trunk build
```

### Backend

Run the backend server:

```bash
doppler run -- cargo run --bin server
```

## Notes

- Axum serves the frontend from `dist/`.
- `trunk build` must be re-run after frontend changes.
- Restart the backend after backend code changes.
- Access the application via Traefik:
  - https://homelab-dev.nixos.allanshomelab.com
