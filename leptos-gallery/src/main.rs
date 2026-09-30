//! Aurora Dark — Leptos component gallery / example app.
//! Consumes the `aurora-leptos` crate (components + tokens + stylesheet) exactly
//! as a downstream route port would.

use aurora_leptos::components::*;
use aurora_leptos::graph::*;
use aurora_leptos::tokens::{token, ApiError};
use aurora_leptos::widgets::*;
use leptos::prelude::*;

fn main() {
    leptos::mount::mount_to_body(App);
}

#[component]
fn Section(
    #[prop(into)] id: String,
    #[prop(into)] title: String,
    #[prop(into)] caption: String,
    children: Children,
) -> impl IntoView {
    view! {
        <section id=id class="gallery__section">
            <div class="gallery__section-title">{title}</div>
            <p class="gallery__caption">{caption}</p>
            {children()}
        </section>
    }
}

/// Left-nav: (anchor id, label). Groups are rendered from the `group` marker.
const NAV: &[(&str, &str)] = &[
    ("§Components", ""),
    ("buttons", "Button"),
    ("layout", "Box · Group · Stack · Text"),
    ("pills", "Pill · StatusBadge · Dot"),
    ("table", "Table"),
    ("forms", "TextInput · Select · Tooltip"),
    ("chips", "Chip"),
    ("panel-modal", "Panel · Modal"),
    ("states", "Loading · Empty · ErrorState"),
    ("more-inputs", "Switch · Number · Password · …"),
    ("layout-prims", "SimpleGrid · Grid · List"),
    ("menu-appshell", "Menu · AppShell"),
    ("§Widgets", ""),
    ("badges", "StateCounts · BuildStatus · HealthPill · Meter"),
    ("banners", "Banner · StaleInputsBanner"),
    ("readiness", "NodeReadiness"),
    ("input-table", "InputTable"),
    ("§Graph", ""),
    ("graph", "Graph / DAG"),
];

#[component]
fn Nav() -> impl IntoView {
    let items = NAV
        .iter()
        .map(|(id, label)| {
            if let Some(group) = id.strip_prefix('§') {
                view! { <div class="gallery-nav__group">{group}</div> }.into_any()
            } else {
                view! { <a class="gallery-nav__link" href=format!("#{id}")>{*label}</a> }.into_any()
            }
        })
        .collect_view();
    view! {
        <nav class="gallery-nav">
            <div class="gallery-nav__title">"Aurora Dark"</div>
            {items}
        </nav>
    }
}

#[component]
fn App() -> impl IntoView {
    let modal_open = RwSignal::new(false);
    let name = RwSignal::new(String::from("nightly-ingest"));
    let bad_field = RwSignal::new(String::new());
    let strategy = RwSignal::new(String::from("when_all"));
    let active_chip = RwSignal::new(0usize);
    // Additional-primitive demo state.
    let switch_on = RwSignal::new(true);
    let segment = RwSignal::new(String::from("Live"));
    let retries = RwSignal::new(3.0_f64);
    let password = RwSignal::new(String::from("hunter2"));
    let notes = RwSignal::new(String::from("Backfill window: 24h"));

    // Demo data. The gallery plays the role of a consuming app (e.g. cloacina):
    // it supplies its own state→label/color vocab; the widgets just render.
    let now = js_sys::Date::now();
    let fresh = Some(now - 5000.0);
    fn state_color(state: &str) -> String {
        match state {
            "live" => token::OK,
            "warming" => token::GOLD,
            "unreachable" => token::BAD,
            _ => token::MUTED,
        }
        .to_string()
    }
    let input = |name: &str, state: &str, last: Option<f64>, total: i64| Input {
        name: name.into(),
        group: Some("rollup".into()),
        state_label: state.replace('_', " "),
        state_color: state_color(state),
        last_event_at: last,
        rate: Some(format!("~{total}/min")),
        ..Default::default()
    };
    let mixed_inputs = vec![
        Input {
            manual_events: Some(3),
            last_manual_event_at: Some(now - 600000.0),
            ..input("orders.events", "live", fresh, 18240)
        },
        Input {
            error: Some("connection refused (econnrefused 10.0.4.12:5432)".into()),
            ..input("inventory.snapshots", "unreachable", Some(now - 9.0e8), 312)
        },
        input("shipments.tracking", "warming", None, 0),
    ];
    let mixed_inputs_table = mixed_inputs.clone();
    let readiness_inputs = vec![
        input("orders.events", "live", fresh, 18240),
        input("clicks.stream", "live", fresh, 92110),
        input("inventory.snapshots", "unreachable", Some(now - 9.0e8), 312),
    ];
    let run_counts = vec![
        StateCount {
            label: "running".into(),
            count: 1,
            color: token::ICE.into(),
        },
        StateCount {
            label: "completed".into(),
            count: 3,
            color: token::OK.into(),
        },
        StateCount {
            label: "failed".into(),
            count: 1,
            color: token::BAD.into(),
        },
        StateCount {
            label: "scheduled".into(),
            count: 1,
            color: token::VIOLET.into(),
        },
    ];
    let g_nodes = vec![
        GraphNode::new("orders", "orders.events")
            .color(token::ICE)
            .sublabel("source"),
        GraphNode::new("clicks", "clicks.stream")
            .color(token::ICE)
            .sublabel("source"),
        GraphNode::new("rollup", "rollup")
            .color(token::VIOLET)
            .sublabel("node"),
        GraphNode::new("warehouse", "warehouse")
            .color(token::TEAL)
            .sublabel("sink"),
    ];
    let g_edges = vec![
        GraphEdge::new("orders", "rollup").active(true),
        GraphEdge::new("clicks", "rollup"),
        GraphEdge::new("rollup", "warehouse"),
    ];
    let g_nodes_lr = g_nodes.clone();
    let g_edges_lr = g_edges.clone();

    let rows = [
        ("exec_7f3a01", "completed", "1.2s", "12:02:11"),
        ("exec_561200", "running", "—", "12:04:53"),
        ("exec_90ab1e", "failed", "0.4s", "11:58:02"),
        ("exec_44c0de", "scheduled", "—", "12:10:00"),
    ];
    let table_rows = rows
        .iter()
        .map(|(run, status, dur, started)| {
            view! {
                <tr>
                    <td>{*run}</td>
                    <td><StatusBadge status=status.to_string() /></td>
                    <td>{*dur}</td>
                    <td>{*started}</td>
                </tr>
            }
        })
        .collect_view();

    let chips = [
        ("All", 128),
        ("Running", 4),
        ("Failed", 11),
        ("Scheduled", 7),
    ];
    let chip_views = chips
        .iter()
        .enumerate()
        .map(|(i, (label, count))| {
            let active = Signal::derive(move || active_chip.get() == i);
            view! {
                <Chip
                    label=label.to_string()
                    count=*count
                    active=active
                    on_click=Callback::new(move |_| active_chip.set(i))
                />
            }
        })
        .collect_view();

    view! {
        <div class="gallery-shell">
            <Nav />
            <main class="gallery-main">
            <PageHeader
                title="Aurora Dark"
                sub="leptos · wasm — aurora-leptos component gallery"
                right=Box::new(|| view! {
                    <Group gap="xs">
                        <StatusBadge status="running" />
                        <HealthPill label="live" color=token::OK tip="Connected and receiving data normally." />
                    </Group>
                }.into_any())
            />
            <p class="gallery__lead">
                "The complete Aurora Dark design system, ported to Leptos 0.8 — every Mantine primitive in use plus all Aurora components, shipped as the aurora-leptos crate."
            </p>

            // ---- Buttons ----
            <Section id="buttons" title="Button" caption="variants: filled · light · default · subtle — sizes xs/sm/md — danger — disabled">
                <Group wrap=true>
                    <Button>"Filled"</Button>
                    <Button variant="light">"Light"</Button>
                    <Button variant="default">"Default"</Button>
                    <Button variant="subtle">"Subtle"</Button>
                    <Button bad=true>"Danger"</Button>
                    <Button variant="light" bad=true>"Danger light"</Button>
                    <Button disabled=true>"Disabled"</Button>
                </Group>
                <Group wrap=true>
                    <Button size="xs">"xs"</Button>
                    <Button size="sm">"sm"</Button>
                    <Button size="md">"md"</Button>
                </Group>
            </Section>

            // ---- Layout + Text ----
            <Section id="layout" title="Box · Group · Stack · Text · MONO" caption="layout primitives + the type helpers">
                <div class="gallery__card">
                    <Stack gap="sm">
                        <Text bright=true bold=true size="lg">"Heading — fg-bright 18/600"</Text>
                        <Text>"Body text — default fg, 16px, line-height 1.55."</Text>
                        <Text dimmed=true size="sm">"Dimmed caption — muted, 14px."</Text>
                        <Text mono=true size="sm">"MONO — 42.7 ops/s · exec_7f3a01 · tabular nums"</Text>
                        <Group gap="sm">
                            <span class="cl-code">"cargo build"</span>
                            <a class="cl-anchor">"Anchor link"</a>
                        </Group>
                    </Stack>
                </div>
            </Section>

            // ---- Pills / StatusBadge / Dot ----
            <Section id="pills" title="Pill · StatusBadge · Dot" caption="status hue at full strength on a 1c-alpha tint — radius 10, Plex Mono">
                <Group wrap=true>
                    <StatusBadge status="running" />
                    <StatusBadge status="completed" />
                    <StatusBadge status="failed" />
                    <StatusBadge status="scheduled" />
                    <StatusBadge status="skipped" />
                    <StatusBadge status="pending" />
                    <StatusBadge status="cancelled" />
                </Group>
                <Group wrap=true>
                    <Pill color=token::ICE>"accumulator"</Pill>
                    <Pill color=token::VIOLET>"reactor"</Pill>
                    <Pill color=token::TEAL>"when_all"</Pill>
                    <Pill color=token::GOLD>"warming"</Pill>
                </Group>
                <Group gap="sm">
                    <Group gap="xs"><Dot color=token::OK /><Text size="sm">"live"</Text></Group>
                    <Group gap="xs"><Dot color=token::ICE glow=true /><Text size="sm">"running (glow)"</Text></Group>
                    <Group gap="xs"><Dot color=token::MUTED size=10 /><Text size="sm">"idle (10px)"</Text></Group>
                </Group>
            </Section>

            // ---- Table ----
            <Section id="table" title="Table" caption="executions — mono cells, hairline rows, uppercase header">
                <div class="gallery__card">
                    <table class="cl-table cl-table--mono">
                        <thead>
                            <tr><th>"run"</th><th>"status"</th><th>"duration"</th><th>"started"</th></tr>
                        </thead>
                        <tbody>{table_rows}</tbody>
                    </table>
                </div>
            </Section>

            // ---- Forms ----
            <Section id="forms" title="TextInput · Select · Tooltip" caption="form controls + hover tooltip">
                <div class="gallery__card">
                    <Stack>
                        <TextInput label="Schedule name" placeholder="e.g. nightly-ingest" value=name />
                        <TextInput label="Cron (invalid)" placeholder="* * * * *" value=bad_field error="Expected 5 fields, got 4" />
                        <Select label="Input strategy" value=strategy options=vec!["when_all".into(), "when_any".into(), "latest".into(), "sequential".into()] />
                        <Group gap="sm">
                            <Tooltip label="Fires the graph only once ALL of its bound accumulators have new data.">
                                <Pill color=token::TEAL>"when_all ⓘ"</Pill>
                            </Tooltip>
                            <Text dimmed=true size="sm">"← hover the pill"</Text>
                        </Group>
                    </Stack>
                </div>
            </Section>

            // ---- Chips ----
            <Section id="chips" title="Chip (filter)" caption="active = ice fill + dark text; inactive = panel + border">
                <Group gap="sm" wrap=true>{chip_views}</Group>
            </Section>

            // ---- Panel + Modal ----
            <Section id="panel-modal" title="Panel · Modal" caption="card surface with section header; overlay modal">
                <Group top=true wrap=true>
                    <div style="flex:1;min-width:320px;">
                        <Panel title="Recent executions" caption="last 24h">
                            <Stack gap="xs">
                                <Group justify="between"><Text mono=true size="sm">"exec_7f3a01"</Text><StatusBadge status="completed" /></Group>
                                <Group justify="between"><Text mono=true size="sm">"exec_561200"</Text><StatusBadge status="running" /></Group>
                                <Group justify="between"><Text mono=true size="sm">"exec_90ab1e"</Text><StatusBadge status="failed" /></Group>
                            </Stack>
                        </Panel>
                    </div>
                    <div style="flex:1;min-width:320px;">
                        <Panel title="Trigger run">
                            <Stack>
                                <Text dimmed=true size="sm">"Manually fire the selected graph now."</Text>
                                <Button on_click=Callback::new(move |_| modal_open.set(true))>"Open modal…"</Button>
                            </Stack>
                        </Panel>
                    </div>
                </Group>
            </Section>
            <Modal open=modal_open title="Confirm manual run">
                <Stack>
                    <Text size="sm">"This will trigger an immediate run of nightly-ingest outside its schedule."</Text>
                    <Group justify="end" gap="sm">
                        <Button variant="default" on_click=Callback::new(move |_| modal_open.set(false))>"Cancel"</Button>
                        <Button on_click=Callback::new(move |_| modal_open.set(false))>"Trigger run"</Button>
                    </Group>
                </Stack>
            </Modal>

            // ---- States ----
            <Section id="states" title="Loading · Empty · ErrorState" caption="every async view uses these — no blank screens, errors render by classified kind">
                <Group top=true wrap=true>
                    <div class="gallery__card" style="flex:1;min-width:240px;"><Loading /></div>
                    <div class="gallery__card" style="flex:1;min-width:240px;"><Empty message="No executions in the last 24 hours." /></div>
                </Group>
                <Group top=true wrap=true>
                    <div style="flex:1;min-width:280px;">
                        <ErrorState error=ApiError::Network />
                    </div>
                    <div style="flex:1;min-width:280px;">
                        <ErrorState error=ApiError::Http { status: 404, message: "No reactor 'r-evt-roll-up' was found in this tenant.".into(), code: None } />
                    </div>
                    <div style="flex:1;min-width:280px;">
                        <ErrorState error=ApiError::Http { status: 422, message: "schedule.cron must be a valid 5-field cron expression.".into(), code: Some("invalid_cron".into()) } />
                    </div>
                </Group>
            </Section>

            // ---- More form controls ----
            <Section id="more-inputs" title="Switch · SegmentedControl · NumberInput · PasswordInput · Textarea · CopyButton · ActionIcon" caption="the rest of the Mantine input primitives">
                <div class="gallery__card">
                    <Stack>
                        <Group gap="sm" wrap=true>
                            <Switch checked=switch_on label="Live updates" />
                            <SegmentedControl value=segment options=vec!["Live".into(), "Paused".into(), "Replay".into()] />
                            <ActionIcon title="Refresh">"⟳"</ActionIcon>
                            <CopyButton value="exec_7f3a01" />
                        </Group>
                        <Group gap="sm" wrap=true>
                            <NumberInput label="Max retries" value=retries />
                            <PasswordInput label="API token" placeholder="paste token" value=password />
                        </Group>
                        <Textarea label="Notes" value=notes rows=3 />
                    </Stack>
                </div>
            </Section>

            // ---- Layout primitives ----
            <Section id="layout-prims" title="Box · SimpleGrid · Grid · List · Table · Divider" caption="layout + structural primitives">
                <div class="gallery__card">
                    <Stack>
                        <SimpleGrid cols=3>
                            <Box><Panel title="orders"><Text size="sm" mono=true>"18,240"</Text></Panel></Box>
                            <Box><Panel title="clicks"><Text size="sm" mono=true>"92,110"</Text></Panel></Box>
                            <Box><Panel title="sessions"><Text size="sm" mono=true>"4,502"</Text></Panel></Box>
                        </SimpleGrid>
                        <Divider />
                        <Grid>
                            <GridCol span=8><Text size="sm">"Grid col span 8 (of 12)"</Text></GridCol>
                            <GridCol span=4><Text size="sm" dimmed=true>"span 4"</Text></GridCol>
                        </Grid>
                        <Divider />
                        <Group top=true wrap=true gap="sm">
                            <div style="flex:1;min-width:220px;">
                                <List>
                                    <ListItem>"Bound accumulators: 3"</ListItem>
                                    <ListItem>"Reaction mode: when_all"</ListItem>
                                    <ListItem>"Input strategy: latest"</ListItem>
                                </List>
                            </div>
                            <div style="flex:2;min-width:280px;">
                                <Table mono=true>
                                    <thead><tr><th>"reactor"</th><th>"fires"</th></tr></thead>
                                    <tbody>
                                        <tr><td>"rollup-reactor"</td><td>"1,204"</td></tr>
                                        <tr><td>"alerts-reactor"</td><td>"87"</td></tr>
                                    </tbody>
                                </Table>
                            </div>
                        </Group>
                    </Stack>
                </div>
            </Section>

            // ---- Menu + AppShell ----
            <Section id="menu-appshell" title="Menu · AppShell" caption="dropdown menu and the app scaffold">
                <Group top=true wrap=true>
                    <Menu label="Actions">
                        <div class="cl-menu__label">"Run"</div>
                        <MenuItem>"Trigger now"</MenuItem>
                        <MenuItem>"Pause schedule"</MenuItem>
                        <div class="cl-menu__divider"></div>
                        <MenuItem>"Delete…"</MenuItem>
                    </Menu>
                    <div style="flex:1;min-width:420px;">
                        <AppShell
                            header=Box::new(|| view! {
                                // app-owned brand mark (the pack ships no branding)
                                <svg width="22" height="22" viewBox="0 0 24 24" fill="none" aria-hidden="true">
                                    <path d="M5 4 C5 12, 12 12, 12 19" stroke="#7fb2ff" stroke-width="1.6" stroke-linecap="round" />
                                    <path d="M12 4 C12 12, 12 12, 12 19" stroke="#5fd0c5" stroke-width="1.6" stroke-linecap="round" />
                                    <path d="M19 4 C19 12, 12 12, 12 19" stroke="#9d8cff" stroke-width="1.6" stroke-linecap="round" />
                                    <circle cx="5" cy="4" r="1.8" fill="#7fb2ff" />
                                    <circle cx="12" cy="4" r="1.8" fill="#5fd0c5" />
                                    <circle cx="19" cy="4" r="1.8" fill="#9d8cff" />
                                    <circle cx="12" cy="20" r="2" fill="#8fbcff" />
                                </svg>
                                <Text bright=true bold=true>"cloacina"</Text>
                            }.into_any())
                            navbar=Box::new(|| view! {
                                <Stack gap="xs">
                                    <Text size="sm" bright=true>"Overview"</Text>
                                    <Text size="sm" dimmed=true>"Executions"</Text>
                                    <Text size="sm" dimmed=true>"Reactors"</Text>
                                    <Text size="sm" dimmed=true>"Settings"</Text>
                                </Stack>
                            }.into_any())
                        >
                            <PageHeader title="Overview" sub="3 nodes · 9 inputs" />
                            <Text dimmed=true size="sm">"Main content area."</Text>
                        </AppShell>
                    </div>
                </Group>
            </Section>

            // ---- Widgets: StateCounts · BuildStatusBadge · HealthPill · Meter ----
            <Section id="badges" title="StateCounts · BuildStatusBadge · HealthPill · Meter" caption="generic data-display widgets — the app supplies labels/colors">
                <Group gap="sm" wrap=true>
                    <StateCounts counts=run_counts />
                </Group>
                <Group gap="sm" wrap=true>
                    <BuildStatusBadge status="success" />
                    <BuildStatusBadge status="building" />
                    <BuildStatusBadge status="failed" />
                    <BuildStatusBadge status="pending" />
                </Group>
                <Group gap="sm" wrap=true>
                    <HealthPill label="live" color=token::OK tip="Connected and receiving data normally." />
                    <HealthPill label="warming" color=token::GOLD tip="Starting up — backfilling before it goes live." />
                    <HealthPill label="unreachable" color=token::BAD tip="Cannot reach the source." />
                </Group>
                <Group gap="sm" wrap=true>
                    <div style="width:160px;"><Meter value=85.0 /></div>
                    <div style="width:160px;"><Meter value=35.0 color=token::GOLD /></div>
                    <div style="width:160px;"><Meter value=10.0 color=token::BAD /></div>
                </Group>
            </Section>

            // ---- Widgets: Banner · StaleInputsBanner ----
            <Section id="banners" title="Banner · StaleInputsBanner" caption="generic callout; stale-inputs convenience built on it">
                <Stack gap="sm">
                    <Banner color=token::ICE icon="ℹ">"Heads up — a generic info banner."</Banner>
                    <Banner>"A warning banner (default gold accent)."</Banner>
                    <Banner color=token::BAD icon="✕">"An error banner."</Banner>
                    <StaleInputsBanner inputs=mixed_inputs />
                </Stack>
            </Section>

            // ---- Widgets: NodeReadiness ----
            <Section id="readiness" title="NodeReadiness" caption="trigger description + per-input freshness + ready/waiting summary">
                <NodeReadiness
                    node="rollup"
                    mode_label="all"
                    strategy_label="latest"
                    require_all=true
                    inputs=readiness_inputs
                    last_run_at=now - 95000.0
                />
            </Section>

            // ---- Widgets: InputTable ----
            <Section id="input-table" title="InputTable" caption="per-input state, last event, rate, freshness Meter, row action">
                <div class="gallery__card">
                    <Panel title="Input freshness" caption="rollup">
                        <InputTable inputs=mixed_inputs_table on_action=Callback::new(|_name: String| {}) action_label="push ▸" />
                    </Panel>
                </div>
            </Section>

            // ---- Graph / DAG ----
            <Section id="graph" title="Graph / DAG" caption="generic node + edge drawing primitives with a built-in layered layout">
                <Group top=true wrap=true gap="sm">
                    <div class="gallery__card" style="overflow:auto;">
                        <div class="gallery__caption">"direction = TB (default)"</div>
                        <Graph nodes=g_nodes edges=g_edges />
                    </div>
                    <div class="gallery__card" style="overflow:auto;">
                        <div class="gallery__caption">"direction = LR"</div>
                        <Graph nodes=g_nodes_lr edges=g_edges_lr direction="LR" />
                    </div>
                </Group>
            </Section>
            </main>
        </div>
    }
}
