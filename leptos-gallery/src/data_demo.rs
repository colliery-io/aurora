//! Gallery sections for the data components (COLLIERY-T-1834): StatTile,
//! Sparkline, SegmentedBar, DetailList, SectionLabel, CodeBlock, LogView,
//! FeedList, RelativeTime, LiveIndicator, Table v2, Pagination; and the
//! input props, Menu v2, Tooltip, CopyButton v2, SecretReveal, AuthCard and
//! the icons.

use std::time::Duration;

use aurora_leptos::components::*;
use aurora_leptos::tokens::token;
use aurora_leptos::widgets::Meter;
use leptos::prelude::*;

use crate::Section;

/// One demo run for the table.
#[derive(Clone, PartialEq)]
struct Run {
    id: String,
    workflow: &'static str,
    status: &'static str,
    duration_ms: f64,
    started: f64,
}

fn demo_runs(now: f64) -> Vec<Run> {
    const WF: [&str; 5] = [
        "nightly-ingest",
        "orders-rollup",
        "billing-export",
        "clicks-sessionize",
        "warehouse-sync",
    ];
    const ST: [&str; 5] = ["completed", "running", "failed", "completed", "scheduled"];
    (0..57u64)
        .map(|i| {
            let h = i.wrapping_mul(2_654_435_761) % 1_000;
            Run {
                id: format!("exec_{:06x}", 0x7f3a01 + i * 4099),
                workflow: WF[(i as usize * 3 + 1) % 5],
                status: ST[(h as usize) % 5],
                duration_ms: (h * 137 % 400_000) as f64 + 420.0,
                started: now - (i as f64) * 97_000.0 - h as f64 * 60.0,
            }
        })
        .collect()
}

#[component]
pub fn DataSections() -> impl IntoView {
    let toaster = use_toaster();
    let now = js_sys::Date::now();

    // ---- LiveIndicator ----
    let live_choice = RwSignal::new(String::from("Live"));
    let live_state = Signal::derive(move || match live_choice.get().as_str() {
        "Live" => LiveState::Live,
        "Connecting" => LiveState::Connecting,
        _ => LiveState::Offline,
    });

    // ---- LogView ----
    let log_seq = RwSignal::new(0u32);
    let levels = ["info", "running", "completed", "failed", "warning"];
    let make_line = move |n: u32| {
        let lv = levels[(n as usize) % levels.len()];
        let color = match lv {
            "info" => token::MUTED,
            "running" => token::ICE,
            "completed" => token::OK,
            "failed" => token::BAD,
            _ => token::GOLD,
        };
        LogLine::new(format!("task step_{n:02} {lv} — rows={}", n * 1_024))
            .time(format!("12:04:{:02}", n % 60))
            .level_color(lv, color)
    };
    let lines = RwSignal::new((0..14).map(make_line).collect::<Vec<_>>());
    log_seq.set(14);
    let add_lines = move |k: u32| {
        let start = log_seq.get_untracked();
        lines.update(|v| v.extend((start..start + k).map(make_line)));
        log_seq.set(start + k);
    };
    let streaming = RwSignal::new(false);
    let stream_handle = StoredValue::new(None::<IntervalHandle>);
    Effect::new(move |_| {
        if streaming.get() {
            let h = set_interval_with_handle(move || add_lines(1), Duration::from_millis(600));
            stream_handle.set_value(h.ok());
        } else if let Some(h) = stream_handle.get_value() {
            h.clear();
            stream_handle.set_value(None);
        }
    });
    on_cleanup(move || {
        if let Some(Some(h)) = stream_handle.try_get_value() {
            h.clear();
        }
    });

    // ---- Table v2 + Pagination ----
    let runs = StoredValue::new(demo_runs(now));
    let sort = RwSignal::new(SortState::by("started", SortDir::Desc));
    let offset = RwSignal::new(0usize);
    let limit = RwSignal::new(10usize);
    let selected = RwSignal::new(None::<String>);
    let show_empty = RwSignal::new(false);
    let total = Signal::derive(move || {
        if show_empty.get() {
            0
        } else {
            runs.with_value(|r| r.len())
        }
    });
    let page_rows = Memo::new(move |_| {
        if show_empty.get() {
            return Vec::new();
        }
        let s = sort.get();
        let mut rows = runs.get_value();
        match s.key.as_deref() {
            Some("id") => rows.sort_by(|a, b| s.order(a.id.cmp(&b.id))),
            Some("workflow") => rows.sort_by(|a, b| s.order(a.workflow.cmp(b.workflow))),
            Some("duration") => {
                rows.sort_by(|a, b| s.order(a.duration_ms.total_cmp(&b.duration_ms)))
            }
            Some("started") => rows.sort_by(|a, b| s.order(a.started.total_cmp(&b.started))),
            _ => {}
        }
        let r = page_range(offset.get(), limit.get(), rows.len());
        rows.into_iter()
            .skip(r.offset)
            .take(limit.get())
            .collect::<Vec<_>>()
    });

    // ---- Inputs ----
    let locked = RwSignal::new(false);
    let saving = RwSignal::new(false);
    let email = RwSignal::new(String::new());
    let secret_pw = RwSignal::new(String::new());
    let board = RwSignal::new(String::new());
    let notes = RwSignal::new(String::new());
    let limit_n = RwSignal::new(5.0_f64);
    let notify = RwSignal::new(true);
    let input_count = RwSignal::new(0u32);
    let change_log = RwSignal::new(String::from("—"));

    // ---- Menu ----
    let last_action = RwSignal::new(String::from("—"));
    let act = move |s: &'static str| Callback::new(move |_| last_action.set(s.to_string()));

    // ---- Secret ----
    let secret = RwSignal::new(String::new());
    let secret_open = RwSignal::new(false);

    // ---- Auth ----
    let signing_in = RwSignal::new(false);
    let auth_email = RwSignal::new(String::from("dylan@colliery.io"));
    let auth_pw = RwSignal::new(String::new());

    let latency = vec![
        42.0, 38.0, 51.0, 47.0, 62.0, 58.0, 71.0, 66.0, 54.0, 49.0, 45.0, 52.0,
    ];
    let throughput = vec![
        120.0, 180.0, 160.0, 240.0, 210.0, 300.0, 280.0, 330.0, 310.0, 360.0, 340.0, 390.0,
    ];
    let errors = vec![0.0, 1.0, 0.0, 0.0, 3.0, 1.0, 0.0, 7.0, 2.0, 0.0, 0.0, 1.0];
    let (lat2, thr2, err2) = (latency.clone(), throughput.clone(), errors.clone());

    view! {
        // ---------------------------------------------------------------- StatTile
        <Section id="stat" title="StatTile · Sparkline" caption="label eyebrow, big value, unit, delta or sub line, status hue, sparkline slot — the sparkline draws in currentColor">
            <div class="gallery-stats" id="demo-stat">
                <StatTile label="Active agents" value="128" sub="of 131 registered" />
                <StatTile label="p95 latency" value="54" unit="ms" delta="−8%" delta_color=token::OK sub="vs last hour"
                    color=token::ICE
                    spark=Box::new(move || view! { <Sparkline values=latency.clone() fluid=true fill=true label="p95 latency, last hour" /> }.into_any()) />
                <StatTile label="Throughput" value="390" unit="/min" delta="+12%" delta_color=token::OK
                    color=token::TEAL
                    spark=Box::new(move || view! { <Sparkline values=throughput.clone() fluid=true label="Throughput, last hour" /> }.into_any()) />
                <StatTile label="Failed runs" value="7" sub="last 24h" color=token::BAD
                    spark=Box::new(move || view! { <Sparkline values=errors.clone() bars=true fluid=true label="Failed runs per hour" /> }.into_any()) />
            </div>
            <Group gap="sm" wrap=true>
                <span class="gallery__label">"line"</span><Sparkline values=lat2 color=token::ICE />
                <span class="gallery__label">"fill"</span><Sparkline values=thr2 fill=true color=token::VIOLET />
                <span class="gallery__label">"bars"</span><Sparkline values=err2 bars=true color=token::GOLD />
                <span class="gallery__label">"flat"</span><Sparkline values=vec![4.0, 4.0, 4.0] color=token::MUTED />
                <span class="gallery__label">"one value"</span><Sparkline values=vec![9.0] color=token::OK />
            </Group>
        </Section>

        // ---------------------------------------------------------------- SegmentedBar
        <Section id="segbar" title="SegmentedBar · Meter" caption="proportions in one bar (ok / warning / failed, done / active / to do); a part has a tooltip; optional legend; a Meter for one value, or indeterminate for work of unknown length">
            <div class="gallery__card" id="demo-segbar">
                <Stack gap="sm">
                    <SegmentedBar label="Fleet health" legend=true segments=vec![
                        Segment::new("healthy", 112.0, token::OK),
                        Segment::new("degraded", 9.0, token::GOLD),
                        Segment::new("failing", 4.0, token::BAD),
                        Segment::new("offline", 3.0, token::MUTED),
                    ] />
                    <SegmentedBar label="Initiative progress" height=6 legend=true segments=vec![
                        Segment::new("done", 14.0, token::TEAL),
                        Segment::new("active", 5.0, token::ICE),
                        Segment::new("to do", 9.0, token::MUTED),
                    ] />
                    <SegmentedBar label="Nothing yet" segments=vec![Segment::new("done", 0.0, token::OK)] />
                    <Group gap="sm">
                        <div style="width:200px;"><Meter value=72.0 label="Disk used" /></div>
                        <span class="gallery__label">"Meter: one value from 0 to 100"</span>
                    </Group>
                    <Group gap="sm">
                        <div style="width:200px;"><Meter indeterminate=true label="Collecting diagnostics" /></div>
                        <span class="gallery__label">"Meter indeterminate=true: work of unknown length (still with reduced motion)"</span>
                    </Group>
                </Stack>
            </div>
        </Section>

        // ---------------------------------------------------------------- DetailList
        <Section id="detail" title="DetailList · KeyValue · SectionLabel" caption="label / value rows with hairline dividers from the tokens; mono values; stacked summary strip; mono uppercase section heading with a count and an action">
            <div class="gallery-two" id="demo-detail">
                <div class="gallery__card">
                    <SectionLabel label="Execution" divider=true
                        action=Box::new(|| view! { <a class="cl-anchor" href="#detail">"View run"</a> }.into_any()) />
                    <DetailList>
                        <KeyValue label="Run" mono=true>"exec_7f3a01"</KeyValue>
                        <KeyValue label="Status"><StatusBadge status="completed" /></KeyValue>
                        <KeyValue label="Started"><RelativeTime at=now - 185_000.0 /></KeyValue>
                        <KeyValue label="Duration" mono=true>{format_duration(4_249.0)}</KeyValue>
                        <KeyValue label="Workflow">"nightly-ingest · a long value wraps onto the next line and does not push the label"</KeyValue>
                    </DetailList>
                </div>
                <div class="gallery__card">
                    <SectionLabel label="Inputs" count=3usize />
                    <DetailList mono=true stacked=true>
                        <KeyValue label="Rows">"18,240"</KeyValue>
                        <KeyValue label="Lag">{format_duration(850.0)}</KeyValue>
                        <KeyValue label="Errors">"0.4%"</KeyValue>
                        <KeyValue label="Dead letters">"12"</KeyValue>
                    </DetailList>
                    <SectionLabel label="No dividers" level=4 />
                    <DetailList dividers=false label_width="90px">
                        <KeyValue label="Owner">"platform-team"</KeyValue>
                        <KeyValue label="Tier">"gold"</KeyValue>
                    </DetailList>
                </div>
            </div>
        </Section>

        // ---------------------------------------------------------------- Code + Log
        <Section id="logs" title="CodeBlock · LogView" caption="mono block on --inset, scrolls, max height, copy; the log has a time and a level pill per line and follows the tail — scroll up and it stops, 'Jump to latest' brings it back">
            <div class="gallery-two" id="demo-logs">
                <CodeBlock max_height="220px" label="Deployment manifest" code="apiVersion: brokkr.io/v1\nkind: Stack\nmetadata:\n  name: orders-rollup\n  labels:\n    tier: gold\nspec:\n  replicas: 3\n  image: registry.colliery.io/orders-rollup:2026.09.30-7f3a01-a-very-long-tag-that-scrolls-sideways\n  env:\n    - name: RUST_LOG\n      value: info\n" />
                <Stack gap="xs">
                    <LogView lines=lines max_height="220px" label="Execution events" />
                    <Group gap="sm">
                        <Button size="xs" variant="default" on_click=Callback::new(move |_| add_lines(5))>"Add 5 lines"</Button>
                        <Switch checked=streaming label="Stream" />
                        <Button size="xs" variant="subtle" on_click=Callback::new(move |_| lines.set(Vec::new()))>"Clear"</Button>
                    </Group>
                </Stack>
            </div>
            <CodeBlock wrap=true copy=false code="cargo install colliery-io-aurora --no-default-features --features bin   # wrap=true, copy=false" />
        </Section>

        // ---------------------------------------------------------------- Feed + time + live
        <Section id="feed" title="FeedList · RelativeTime · LiveIndicator" caption="time, dot, subject, actor, status pill, text, click-through; '3m ago' in a <time> with the full time as a tooltip, one shared tick; live / connecting / offline — the dot pulses only when live">
            <div class="gallery__card" id="demo-feed">
                <Group justify="between" wrap=true>
                    <SectionLabel label="Live activity" />
                    <Group gap="sm" wrap=true>
                        <SegmentedControl value=live_choice options=vec!["Live".into(), "Connecting".into(), "Offline".into()] />
                        <LiveIndicator state=live_state />
                        <LiveIndicator state=live_state compact=true />
                    </Group>
                </Group>
                <FeedList label="Recent events">
                    <FeedRow at=now - 3_000.0 subject="orders-rollup" status="running" dot=token::ICE
                        on_click=Callback::new(move |_| { toaster.info("Open orders-rollup"); })>"Run started by the schedule"</FeedRow>
                    <FeedRow at=now - 42_000.0 subject="nightly-ingest" actor="dylan" status="completed" dot=token::OK
                        href="#table2">"Completed in 4.2s · 18,240 rows"</FeedRow>
                    <FeedRow at=now - 185_000.0 subject="billing-export" status="failed" dot=token::BAD
                        on_click=Callback::new(move |_| { toaster.error("Open billing-export"); })>"connection refused (econnrefused 10.0.4.12:5432) — a long line is cut with an ellipsis and does not wrap"</FeedRow>
                    <FeedRow at=now - 7_400_000.0 subject="COLLIERY-T-1834" actor="claude-code" status="review" status_color=token::VIOLET>"Moved to Review"</FeedRow>
                    <FeedRow time="Sep 12" subject="warehouse-sync" status="paused">"Paused by an operator"</FeedRow>
                </FeedList>
                <div style="height:12px;"></div>
                <Group gap="sm" wrap=true>
                    <span class="gallery__label">"RelativeTime:"</span>
                    <RelativeTime at=now - 1_000.0 />
                    <RelativeTime at=now - 3_600_000.0 * 5.0 />
                    <RelativeTime iso="2026-01-02T03:04:05Z" />
                    <RelativeTime at=now + 180_000.0 />
                    <RelativeTime iso="not a time" />
                    <span class="gallery__label">{format!("format_duration: {} · {} · {} · {}", format_duration(850.0), format_duration(4_249.0), format_duration(185_000.0), format_duration(7_440_000.0))}</span>
                </Group>
            </div>
        </Section>

        // ---------------------------------------------------------------- Table v2
        <Section id="table2" title="Table v2 · Pagination" caption="clickable rows (mouse, Enter, Space; a button in the row does not open it), sortable header cells with aria-sort, fixed layout with column widths, an empty row; Prev / Next, '41–60 of 212', rows per page">
            <div class="gallery__card" id="demo-table">
                <Group justify="between">
                    <Text size="sm" dimmed=true>
                        {move || match selected.get() {
                            Some(id) => format!("Opened: {id}"),
                            None => "Click a row, or Tab to it and press Enter.".to_string(),
                        }}
                    </Text>
                    <Switch checked=show_empty label="Show empty" />
                </Group>
                <Table fixed=true label="Runs" min_width="640px"
                    widths=vec!["22%".into(), "26%".into(), "16%".into(), "14%".into(), "14%".into(), "8%".into()]>
                    <thead>
                        <tr>
                            <SortHeader label="Run" key="id" sort=sort />
                            <SortHeader label="Workflow" key="workflow" sort=sort />
                            <th>"Status"</th>
                            <SortHeader label="Duration" key="duration" sort=sort first_desc=true align_right=true />
                            <SortHeader label="Started" key="started" sort=sort first_desc=true />
                            <th class="cl-num">"Copy"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || {
                            let rows = page_rows.get();
                            if rows.is_empty() {
                                return view! { <TableEmpty colspan=6 message="No runs match these filters." /> }.into_any();
                            }
                            rows.into_iter().map(|r| {
                                let id = r.id.clone();
                                let id_sel = r.id.clone();
                                let id_copy = r.id.clone();
                                view! {
                                    <TableRow
                                        label=format!("Open {}", r.id)
                                        selected=Signal::derive(move || selected.get().as_deref() == Some(id_sel.as_str()))
                                        on_click=Callback::new(move |_| selected.set(Some(id.clone())))
                                    >
                                        <td class="cl-mono">{r.id.clone()}</td>
                                        <td>{r.workflow}</td>
                                        <td><StatusBadge status=r.status /></td>
                                        <td class="cl-num cl-mono">{format_duration(r.duration_ms)}</td>
                                        <td><RelativeTime at=r.started /></td>
                                        <td class="cl-num"><CopyButton value=id_copy icon=true label="Copy run id" /></td>
                                    </TableRow>
                                }
                            }).collect_view().into_any()
                        }}
                    </tbody>
                </Table>
                <Pagination offset=offset limit=limit total=total page_sizes=vec![10, 20, 50] />
            </div>
        </Section>

        // ---------------------------------------------------------------- Inputs
        <Section id="inputs2" title="Input and button props" caption="disabled from a signal, loading on Button (spinner, disabled, aria-busy), on_input / on_change, name, autocomplete, required, Select (value, label) pairs with a placeholder, NumberInput min / max">
            <div class="gallery__card" id="demo-inputs">
                <Stack>
                    <Group gap="sm" wrap=true>
                        <Switch checked=locked label="Lock the form" />
                        <Button attr:data-testid="demo-save" loading=saving loading_label="Saving…" disabled=locked
                            on_click=Callback::new(move |_| {
                                saving.set(true);
                                set_timeout(move || saving.set(false), Duration::from_millis(1800));
                            })>"Save"</Button>
                        <Button variant="default" loading=saving disabled=locked>"Loading, no label"</Button>
                        <Button variant="light" bad=true disabled=move || locked.get() || email.get().is_empty()>"Needs an email"</Button>
                        <Button variant="subtle" href="#inputs2">"A link that looks like a button"</Button>
                    </Group>
                    <div class="gallery-two">
                        <TextInput label="Email" value=email input_type="email" name="email"
                            autocomplete="username" required=true spellcheck=false placeholder="you@example.com"
                            disabled=locked
                            on_input=Callback::new(move |_s: String| input_count.update(|n| *n += 1))
                            on_change=Callback::new(move |s: String| change_log.set(format!("email = {s}"))) />
                        <PasswordInput label="Password" value=secret_pw name="password"
                            autocomplete="current-password" required=true disabled=locked />
                        <Select label="Board" value=board placeholder="Choose a board" disabled=locked
                            option_pairs=vec![
                                ("12".into(), "COLLIERY · Delivery".into()),
                                ("14".into(), "COLLIERY · Strategy".into()),
                                ("31".into(), "KAIROS · Archive".into()),
                            ]
                            on_change=Callback::new(move |s: String| change_log.set(format!("board = {s}"))) />
                        <NumberInput label="Limit (1–10)" value=limit_n min=1.0 max=10.0 disabled=locked
                            on_change=Callback::new(move |n: f64| change_log.set(format!("limit = {n}"))) />
                        <Textarea label="Notes" value=notes rows=2 mono=true disabled=locked />
                        <Stack gap="xs">
                            <Switch checked=notify label="Notify me" disabled=locked
                                on_change=Callback::new(move |b: bool| change_log.set(format!("notify = {b}"))) />
                            <TextInput label="With an error" value=RwSignal::new(String::from("nightly ingest")) error="Use letters, digits and dashes." />
                        </Stack>
                    </div>
                    <Text size="sm" dimmed=true mono=true>
                        {move || format!("on_input calls: {} · last on_change: {} · board value: {:?}", input_count.get(), change_log.get(), board.get())}
                    </Text>
                </Stack>
            </div>
        </Section>

        // ---------------------------------------------------------------- Menu + Tooltip
        <Section id="menu2" title="Menu v2 · Tooltip" caption="custom trigger, right alignment, opens up; Enter / Space / ArrowDown open it, arrows move, Escape and a click outside close it; Tooltip: placement, delay, keyboard focus shows it, Escape hides it">
            <div class="gallery__card" id="demo-menu">
                <Stack gap="sm">
                <Group gap="sm" wrap=true top=true>
                    <Menu label="Actions">
                        <MenuLabel>"Run"</MenuLabel>
                        <MenuItem on_click=act("Trigger now")>"Trigger now"</MenuItem>
                        <MenuItem on_click=act("Pause schedule")>"Pause schedule"</MenuItem>
                        <MenuItem disabled=true on_click=act("Disabled")>"Resume (disabled)"</MenuItem>
                        <MenuDivider />
                        <MenuItem danger=true on_click=act("Delete")>"Delete…"</MenuItem>
                    </Menu>
                    <div style="width:220px;">
                        <Menu trigger=Box::new(|| view! {
                            <Dot color=token::OK size=7 />
                            <span style="flex:1;font-family:var(--font-mono);">"acme / prod"</span>
                            <IconChevron size=14 />
                        }.into_any())>
                            <MenuItem on_click=act("acme / prod")>"acme / prod"</MenuItem>
                            <MenuItem on_click=act("acme / staging")>"acme / staging"</MenuItem>
                            <MenuDivider />
                            <MenuItem on_click=act("Add tenant")>"+ Add tenant"</MenuItem>
                        </Menu>
                    </div>
                    <div style="margin-left:auto;">
                        <Menu aria_label="More" align="end"
                            trigger_class="cl-action-icon"
                            trigger=Box::new(|| view! { <IconMenu /> }.into_any())>
                            <MenuItem on_click=act("Copy link")>"Copy link"</MenuItem>
                            <MenuItem href="#menu2" on_click=act("Open")>"Open in place"</MenuItem>
                        </Menu>
                    </div>
                </Group>
                <Text size="sm" dimmed=true mono=true>{move || format!("last action: {}", last_action.get())}</Text>
                <Group gap="sm" wrap=true>
                    <Tooltip label="Top (default): shows after a short delay." focusable=true><Pill color=token::TEAL>"top"</Pill></Tooltip>
                    <Tooltip label="Bottom." position="bottom" focusable=true><Pill color=token::ICE>"bottom"</Pill></Tooltip>
                    <Tooltip label="Left." position="left" focusable=true><Pill color=token::VIOLET>"left"</Pill></Tooltip>
                    <Tooltip label="Right: on an inline SVG icon." position="right" focusable=true>
                        <span style="color:var(--muted);display:inline-flex;"><IconInfo size=18 /></span>
                    </Tooltip>
                    <Tooltip label="On a button: keyboard focus shows it.">
                        <Button size="xs" variant="default">"Focus me"</Button>
                    </Tooltip>
                </Group>
                </Stack>
            </div>
        </Section>

        // ---------------------------------------------------------------- Copy + Secret
        <Section id="copy" title="CopyButton v2 · SecretReveal" caption="text or icon; copies a link; not there without the Clipboard API; an aria-live region says 'Copied'. SecretReveal shows a secret one time in a modal">
            <div class="gallery__card" id="demo-copy">
                <Group gap="sm" wrap=true>
                    <CopyButton value="exec_7f3a01" />
                    <CopyButton value="exec_7f3a01" icon=true label="Copy run id" />
                    <CopyButton value="/items/COLLIERY-T-1834" link=true label="Copy link"
                        on_copy=Callback::new(move |s: String| { toaster.success(format!("Copied {s}")); }) />
                    <Code>"/items/COLLIERY-T-1834"</Code>
                    <span style="flex:1"></span>
                    <Button on_click=Callback::new(move |_| {
                        secret.set("aur_live_7f3a01c9d2e84b1f0a6e3d5c2b9a8f7e6d5c4b3a2f1e0d9c".into());
                        secret_open.set(true);
                    })>"Create an API key"</Button>
                </Group>
            </div>
            <Modal open=secret_open title="API key created" close_on_scrim=false locked=Signal::derive(|| true)>
                <SecretReveal secret=secret label="API key"
                    on_done=Callback::new(move |_| {
                        secret_open.set(false);
                        secret.set(String::new());
                        toaster.success("The key is saved. It is not shown again.");
                    }) />
            </Modal>
        </Section>

        // ---------------------------------------------------------------- Auth
        <Section id="auth" title="CenterScreen · AuthCard" caption="a centred card for sign-in, the callback and gate states (shown contained here; in a product it fills the screen)">
            <div class="gallery-two" id="demo-auth">
                <CenterScreen contained=true>
                    <AuthCard title="Sign in" sub="Use your Colliery account."
                        brand=Box::new(|| view! {
                            <span style="color:var(--ice);display:inline-flex;"><IconBolt size=20 /></span>
                            <span style="font-weight:600;color:var(--fg-bright);">"Aurora"</span>
                        }.into_any())
                        footer=Box::new(|| view! { "Trouble signing in? " <a class="cl-anchor" href="#auth">"Get help"</a> }.into_any())>
                        <TextInput label="Email" value=auth_email input_type="email" name="email" autocomplete="username" required=true disabled=signing_in />
                        <PasswordInput label="Password" value=auth_pw name="password" autocomplete="current-password" required=true disabled=signing_in />
                        <Button loading=signing_in loading_label="Signing in…"
                            on_click=Callback::new(move |_| {
                                signing_in.set(true);
                                set_timeout(move || signing_in.set(false), Duration::from_millis(1800));
                            })>"Sign in"</Button>
                    </AuthCard>
                </CenterScreen>
                <CenterScreen contained=true>
                    <AuthCard title="Checking your session" sub="This takes a moment after the sign-in page sends you back.">
                        <Loading label="Signing you in…" />
                    </AuthCard>
                </CenterScreen>
            </div>
        </Section>

        // ---------------------------------------------------------------- Icons
        <Section id="icons" title="Icons" caption="a small SVG set in currentColor: the icon takes the colour of its parent; with a title it is role=img, without one it is hidden from screen readers">
            <div class="gallery__card gallery-icons" id="demo-icons">
                <span class="gallery-icon"><IconPlay size=18 />"play"</span>
                <span class="gallery-icon"><IconPause size=18 />"pause"</span>
                <span class="gallery-icon"><IconBolt size=18 />"bolt"</span>
                <span class="gallery-icon"><IconCopy size=18 />"copy"</span>
                <span class="gallery-icon"><IconClose size=18 />"close"</span>
                <span class="gallery-icon"><IconChevron size=18 />"chevron"</span>
                <span class="gallery-icon"><IconChevron size=18 dir="right" />"chevron right"</span>
                <span class="gallery-icon"><IconExternal size=18 />"external"</span>
                <span class="gallery-icon"><IconCheck size=18 />"check"</span>
                <span class="gallery-icon"><IconAlert size=18 />"alert"</span>
                <span class="gallery-icon"><IconInfo size=18 />"info"</span>
                <span class="gallery-icon"><IconMenu size=18 />"menu"</span>
                <span class="gallery-icon"><IconSearch size=18 />"search"</span>
                <span class="gallery-icon"><IconSun size=18 />"sun"</span>
                <span class="gallery-icon"><IconMoon size=18 />"moon"</span>
                <span class="gallery-icon"><IconMonitor size=18 />"monitor"</span>
                <span class="gallery-icon" style="color:var(--ok);"><IconCheck size=18 title="Done" />"in a hue"</span>
                <button class="cl-action-icon" aria-label="Run"><IconPlay /></button>
                <Button size="xs" variant="light"><IconBolt size=14 />"Fire"</Button>
            </div>
        </Section>
    }
}
