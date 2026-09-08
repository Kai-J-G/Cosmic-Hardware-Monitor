# Screenshots

These three files are referenced by `data/io.github.kai_j_g.CosmicHardwareMonitor.metainfo.xml`
and are **required for a Flathub submission** — the build validates that every
screenshot URL resolves.

| File | What to capture |
| --- | --- |
| `overview.png` | The popup's overview, expanded, showing all four dials |
| `cpu.png` | The CPU tab with the per-core grid open |
| `gpu.png` | The GPU tab |

Flathub asks for PNG, at least 620px wide, in a 16:9-ish aspect. Capture the
popup only, not the whole desktop, and grab them while something is loading the
CPU or GPU so the thermal colour-coding is actually visible.

Once added, re-run:

```bash
appstreamcli validate data/io.github.kai_j_g.CosmicHardwareMonitor.metainfo.xml
```
