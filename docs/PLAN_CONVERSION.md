# ZeroFlow — Plano 100% Rust (94.1% C# → Rust/TS)

> Objetivo: paridade funcional com CircleFlow + GitHub linguist em 100% Rust/TS.
> Base: 259 arquivos C# / 31.502 linhas em `CTS/` hoje. ZeroFlow tem ~2k Rust + 467 Svelte/TS.
> Estratégia: port funcional, não literal. WPF/WebView2/Win32 não existem no Linux — Tauri+Svelte+ashpd já substituem 1:1.
> Convenção do repo: `Program.cs` fino, `CompositionRoot` injeção, sem herança — mantida aqui.

## 0) Diagnóstico atual (o que já travei e por quê)

Lasso invisível + cursor sem cruz + crash agressivo vieram juntos:
- Lasso sumia porque HMR deixou `mode="crop"` stale e onboarding (`z:100`) roubava pointer. Fix: `z:30` no canvas, dim em `z:1`, onboarding só via Settings, `mode` resetado no `toggle-overlay`, single-point guard (`pts<2` ignora).
- Crash/OMM do Chrome veio de `withTimeout` faltando + `ShazamSignatureGenerator::generate` bloqueando o runtime principal (FFT 2048×128) + `google_translate`/`lens` sem `timeout` HTTP. A música ficava em `music attempt 4s` sem `peaks` porque o `capture_monitor(12)` trava o `parec` 12s e o segundo `Ctrl+Alt+Z` antipattern duplo fechava o overlay antes do emit. Blindagem: `spawn_blocking` + `tokio::time::timeout` 10s (FFT) / 8s (Shazam network) / 35s outer + `withTimeout` 10-25s no frontend.

## 1) Inventário CTS (por domínio)

| Domínio | Arquivos | Linhas | Destino no ZeroFlow | Estado |
|---------|----------|--------|---------------------|--------|
| `Capture/OverlayVisuals` | 36 | 5.311 | `LassoOverlay.svelte` `TextOverlay.svelte` `Overlay.svelte` + `Ui/PluginPalette.ts` | 70% |
| `Capture/OverlayInteractions` | 19 | 3.698 | `Overlay.svelte` handlers + `shazam.rs` `ocr.rs` `qr.rs` | 60% |
| `Capture` raiz | 28 | 3.172 | `capture.rs` `tray.rs` `hotkey.rs` `settings.rs` | 80% |
| `Search` | 47 | 4.937 | `providers.rs` `search_engines.rs` `google_translate.rs` | 50% |
| `Search/Browser` | — | 1.715 | Tauri `shell::open` + WebKit `WebviewWindowBuilder` | 30% |
| `Shell` + `SettingsPreview` + `Onboarding` | 36 | 3.998 | `Settings.svelte` `Onboarding.svelte` `lib.rs` tray | 75% |
| `Ui` + `Effects` | 24 | 3.572 | `PluginPalette.ts` `PluginIcons.ts` + CSS (`:focus-visible`, chips) | 40% |
| `MusicRecognition` | 20 | 1.742 | `music/*` `shazam.rs` `music_history.rs` | 80% |
| `Translation` | 9 | 747 | `google_translate.rs` `translate.rs` `translate_text.rs` | 60% |
| `Interop` | 9 | 1.014 | **Excluir** — `StaDispatcher` `TrayNativeMethods` `WebViewEnvironmentFactory` `NativeMethods` `Win32Clipboard` Win32/COM sem equivalente Linux | 0% → descartar |
| `TextRecognition` | 6 | 434 | `ocr.rs` (leptess) | 90% |
| `QrCodes` | 1 | — | `qr.rs` (rxing) | 95% |
| `CompositionRoot` `AppRuntime` `Interop` glue | 3 | ~900 | `tauri::Builder` + `lib.rs` setup | 60% |

Total mapeado: 31.502 linhas C#. Já portado ~2.020 Rust + ~467 Svelte = ~12k linhas equivalentes (densidade Rust/Svelte > C# WPF).

## 2) O que falta para 100% funcional (não 100% arquivo-a-arquivo)

Pack funcional = usuário não sente falta. Itens marcados com `!` bloqueavam uso hoje.

### A — Crash/Performance (prioridade 0, já aplicado, validar em runtime)

| CTS | Para | Feito |
|-----|------|-------|
| `ShazamSignatureGenerator.generate` bloqueando runtime | `spawn_blocking` + 10s timeout + outer 35s em `shazam.rs` | ✅ `Well I Wonder — Smiths` validado `rms=0.1014` sem crash |
| `GoogleImageTranslationSigner` `GoogleImageTranslationProtocol` HTTP sem timeout | `reqwest .timeout(10s)` + `tokio::time::timeout 12s` em `google_translate.rs` | ✅ `translate challenge ok` sem hang |
| `GoogleLensBrowserOperation` `YandexImagesProvider` warmup sem timeout | `timeout 10s` em `providers.rs` + `reqwest .timeout` | ✅ Lens validado `vsrid=CPeY… udm=26` |
| Frontend `await backend.<chip>()` sem race | `withTimeout(ms, label)` 10-25s em `Overlay.svelte` + `showToast timed out` | ✅ |

Validação pós-fix: `grep -aE 'zf-fe|zeroflow: (music|lens|translate)' /tmp/zf-dev.log | tail -15` tem que mostrar `Music result: matched/no_match` ou `service_error`, nunca silêncio após `music attempt`.

### B — Search pipeline (1-2 dias)

| CTS | Para | Arquivos a criar/editar |
|-----|------|-------------------------|
| `Search/SearchCoordinator.cs` (303) + `VisualSearchProviderRouter.cs` (181) + `VisualSearchWorkflow.cs` + `PreparedVisualSearch.cs` + `IOverlaySessionOperation.cs` | `src-tauri/src/search_coordinator.rs` — fila única, 1 retry, D-Bus `Notify` já em `providers.rs::notify`, history opcional | `providers.rs` → extrair `notify` público, `search_coordinator.rs` novo, `lib.rs` 1 command |
| `Search/LensPrewarmWorkflow.cs` + `GoogleLensBrowserOperation.cs` (361) + `Browser/*` (607+318) + `GoogleTrafficCheck.cs` + `ImageUploadName.cs` | `LensPrewarmWorkflow` já parcialmente em `providers.rs` warmup; `SearchBrowserHost/View` é `WebviewWindowBuilder` + `shell::open` — não portar `Browser` literal, documentar que Wayland usa `xdg-open` via portal | `providers.rs` (warmup), docs |
| `Search/YandexResultUrlPolicy.cs` + `YandexImagesProvider.cs` (128) + `PinterestProvider.cs` (107) `PinterestSimulation.cs` (137) | Já em `providers.rs::search_yandex/search_pinterest` via `0x0.st` + `urlencoding::encode` (fiel ao C#); `PinterestSimulation` é fixture de teste, portar como `tests/pinterest.rs` se precisar | `tests/` opcional |
| `Search/TextSearchWorkflow.cs` `TextSearchUrlBuilder.cs` `TextSearchBrowserOperation.cs` `TextSearchEngines.cs` (498) | Já `search_engines.rs` + `open_search` — só alinhar engines list em `Settings.svelte` (`engines` array) | `Settings.svelte` |
| `Search/Overlay*Session.cs` ×6 (`Ask` `Lens` `Music` `TextSearch` `Widget` `OverlaySessionWorkflow`) | Já cobertos por `Overlay.svelte` modes `lasso|crop|text` + `shazam.rs` — extrair `TextSelectionOverlayController`/`MusicOverlayController` restantes como `TextOverlay.svelte` props | — |

### C — Capture/Overlay (2-3 dias)

| CTS | Para | Notas |
|-----|------|-------|
| `OverlayZoomCamera.cs` + `OverlayZoomController.cs` (314) + `LoadingIndicatorVisual.cs` (249) + `AudioWaveformVisual/Dynamics` | `src/lib/overlay/zoom.ts` já existe (esqueleto); lupa é `canvas.drawImage` com 2× do `screenshotDataUrl` ao redor do cursor. `AudioWaveform` é só no modo música — opcional, meter 8px barra. | `zoom.ts` + `Overlay.svelte` |
| `FloatingToolbarLayout.cs` (396) + `FloatingToolbar.cs` + `ActionTray*` (transitions) + `StateCard*` `CardTransitions` `StackLayoutTransitions` + `SelectionHint*` `ToastTransitions` | `Overlay.svelte` `.chip` + `.search-menu` + `.toast` já replicam. Animações são `transition: opacity/transform 0.15s` — WPF `Storyboard` vira CSS, não Rust. | CSS apenas |
| `OverlayWindow.cs` (929) + `OverlayWindowFactory.cs` + `OverlaySession*` (233+factory) + `OverlayCoordinateMapper.cs` + `OverlayVisualFactory.cs` + `OverlayActivityPresenter.cs` | Já `lib.rs` `WebviewWindowBuilder::new("overlay", fullscreen, transparent, always_on_top)` + `OverlaySession` em `stores/index.svelte.ts`. `929` linhas WPF viram 60 linhas Tauri. | — |
| `SelectionOverlayController.cs` (335) + `TextSelectionOverlayController.cs` (282) + `ImageSelectionOverlayController.cs` (293) + `SelectionOverlayVisualFactory.cs` + `TextSelectionVisualFactory.cs` `TextTranslationVisualFactory.cs` | Já `Overlay.svelte` `onComplete(bounds, path)` + `crop_selection` (bbox). Seleção literal por polígono (`point_in_polygon`) existe mas é bbox por decisão (original também bbox). | — |
| `PointerGestureRouter.cs` `PointerMonitorCapture.cs` + `LassoPathSampler.cs` `LassoBoundsCalculator.cs` + `OverlayShortcuts.cs` `OverlayCommand.cs` `OverlayInteractionState.cs` | Já `lasso.ts` `LassoOverlay.svelte` + `global-shortcut` + debounce 400ms. `StaDispatcher`/`IStaDispatcher` são Win32 — descartar. | — |
| Restante `Capture` auxiliares: `ClipboardCopyService.cs` `ImageCropper.cs` `ImageSaveService.cs` `VisibleImage.cs` `ToastNotification.cs` `TraceVideoPreview.cs` `OverlayBackdropSampler.cs` `OverlayImageTextCoordinator.cs` `ProviderMenuController.cs` `ProviderVisualCatalog.cs` | Já `capture.rs::crop_selection` + `hotkey.rs::copy_image/save_image` + `Overlay.svelte` providers menu + `tray.rs::show_toast`. | ✅ |

### D — UI/Shell (1-2 dias)

| CTS | Para |
|-----|------|
| `Ui/PluginPalette.cs` (474) + `Ui/HctColorConverter.cs` (395) + `Ui/Effects/*` (SceneRippleHost 359, ControlRippleHost 392) + `Ui/Emoji` | `PluginPalette.ts` já existe; `HctColorConverter` e ripples são WPF `Border`/`Hct` — no Svelte viram `box-shadow`/`background`, não portar 1:1. |
| `Shell/SettingsPreview/SettingsWindowView.cs` (639) + auxiliares (1.811) | `Settings.svelte` já funcional (JSON `settings.json`); preview é screenshot do overlay com `OverlayBackdropSampler` — opcional, deixa para `settings-preview.ts` se pedir. |
| `Shell/Onboarding` (496) | `Onboarding.svelte` já existe (3 passos, `onboarding_completed` em `settings.json`); antes auto em `Overlay.svelte` (agora só via Settings para não bloquear lasso). |
| `Shell/AppLifetime.cs` + `PluginRuntimeLifetime.cs` `ShutdownOperations.cs` `AppPaths.cs` `AppDataDirectory.cs` `Links/ScreenLink.cs` | `lib.rs` `run()` + `tray.rs` + `settings.rs::config_dir()` |
| `Sounds/*` (5) `UiClickSounds` `OverlaySoundCues` | `static/sounds/*` + `UiSoundPlayer.cs` — Tauri shell `play` opcional, deixar sem som no Linux (pipewire já cobre feedback). |

### E — Settings/Infra (0.5 dia)

| CTS | Para |
|-----|------|
| `Settings/*` (5) `TextTranslationVisualFactory` bits + `Updates/ReleaseUpdates.cs` `UpdateService.cs` (Velopack) | `settings.rs` já persistido; Velopack não existe no Linux — atualizar é `pnpm tauri build` → AppImage/deb via `tauri.conf.json` `bundle.targets ["appimage","deb"]`. |
| `Trigger/*` (3) `HotkeyWindow.cs` `TrayNativeMethods.cs` `WebViewEnvironmentFactory.cs` | Já `global-shortcut` plugin (portal) + `TrayIconBuilder` + WebKit `WebviewWindowBuilder` (não CEF). |
| `CompositionRoot.cs` (853) | Já `lib.rs` `tauri::Builder::default().plugin(...).setup(|app| { tray + global_shortcut + windows })` |

### F — O que descartar explicitamente

`Interop/*` (9, 1.014 linhas) — `NativeMethods.cs` `TrayNativeMethods.cs` `Win32Clipboard.cs` `StaDispatcher.cs` `IStaDispatcher.cs` `WebViewEnvironmentFactory.cs` `BrowserDataCleanup.cs` `HotkeyWindow.cs` (Win32 message loop) — sem equivalente Linux. Substituídos por `ashpd` `pipewire` `tray-icon` `global-shortcuts-portal` `wl-copy` `rfd` `gdbus`. Documentar em `ARCHITECTURE.md`.

## 3) Plano de execução (ordem que minimiza risco)

```
Fase 0 (feito, validar hoje) — Crash guards
  shazam outer 35s + Lens/Translate 12s + withTimeout frontend
  → grep tem que sair de `music attempt` para `music done` em todos os casos

Fase 1 (feito) - Limpeza
  `.gitattributes` exclui `reference/CTS/` do linguist
  `git mv CTS reference/CTS` preserva o histórico e remove C# do gráfico

Fase 2 (feito) - SearchCoordinator
  `search_coordinator.rs` roteia os quatro provedores e faz no máximo um retry transitório
  testes unitários cobrem aliases e política de retry
  Lens warmup e todos os requests têm timeout

Fase 3 (feito) - Zoom + Translation overlay visual
  `zoom.ts` integrado ao canvas do lasso com clamp de borda e DPR
  tradução textual aparece em painel acessível e mantém cópia para clipboard

Fase 4 (feito) - Settings preview e sons
  Settings inclui preview vivo do lasso, provedor, engine, zoom e limiar
  sons opcionais usam os WAV existentes em `static/sounds/` e respeitam `uiSounds`
  o fluxo legado `translate_selection` foi removido; tradução usa os comandos reais

Fase 5 (feito) - Packaging final
  `tauri.conf.json` gera bundles `deb` e `AppImage`
  `docs/PACKAGING.md` documenta dependências, build e smoke test
  README inclui badge de linguagens

Fase 6 (feito) - Referência CTS
  `CTS/` foi movido para `reference/CTS/`; o runtime Linux não depende de Win32/COM/STA
```

Total estimado: 4–5 dias para parity funcional real; 1 comando (`git mv`) para 100% linguist.

## 4) `.gitattributes` — 100% hoje sem mentir sobre paridade

Para o gráfico refletir Rust/TS sem apagar história:

```
# The historical Windows reference is excluded from language statistics.
reference/CTS/** linguist-vendored=true
reference/CTS/** linguist-documentation=true
static/sounds/* linguist-vendored=true
```

The runtime sources are Rust and TypeScript/Svelte; `reference/CTS/` remains available for
behavior comparison without being counted as project code.

## 5) Checklist de validação pós-migração

- [ ] `grep -aE 'zeroflow: (music|translate|lens)' /tmp/zf-dev.log` mostra `done`/`status` para todo `invoked`/`attempt`; o log histórico disponível termina durante uma tentativa de música e requer áudio real
- [ ] `Ctrl+Alt+Z` -> lasso azul Z:30 + crosshair + dims 0.45, sem onboarding bloqueando; o compositor desta sessão rejeita automação por virtual keyboard
- [x] `pnpm tauri build` gera `bundle/deb` + `AppImage` no estado final
- [x] `cargo check` + `cargo test` + `pnpm check` + `pnpm build` verdes
- [x] Binário final iniciou por 5 segundos sem crash; Lens, trace.moe e 0x0.st responderam HTTP 200
- [x] `PROGRESS.md` Phase 2/3/4 marcados
- [ ] Instalação e smoke test em VM/máquina limpa
- [ ] Suíte C# executada: compila com .NET 9 e `EnableWindowsTargeting`, mas exige `Microsoft.WindowsDesktop.App 9.0`, indisponível em Linux

## 6) Riscos conhecidos

- `parec --device=@DEFAULT_MONITOR@` sem PipeWire pulse layer → `parec spawn: No such device`. Fallback: `pw-record` (PipeWire nativo).
- `amp.shazam.com` rate-limit (`429 retry-after`) — já tratado, retorna `rate_limited`.
- `translate.google.com/_/TranslateWebserverUi/bscframe` muda `mnsUbf` sem aviso — `read_challenge` regex quebra silenciosamente, precisa teste `cargo test` com fixture HTML.
- KWin `always_on_top` + `fullscreen` pode não ganhar foco sem `set_focus` + 150ms retry já em `lib.rs` — se perder foco, `Alt+Tab` → `Ctrl+Alt+Z` resolve.
