//! Gallery sections for the frame: AppShell + SideNav, PageHeader, Modal +
//! Drawer, ConfirmDialog, Toast, Tabs and Card.

use std::time::Duration;

use aurora_leptos::components::*;
use aurora_leptos::theme::ThemeToggle;
use aurora_leptos::tokens::token;
use aurora_leptos::widgets::Banner;
use leptos::prelude::*;

use crate::Section;

/// The app's own brand mark (the pack ships no branding).
#[component]
fn DemoBrand(#[prop(into)] name: String) -> impl IntoView {
    view! {
        <svg width="22" height="22" viewBox="0 0 24 24" fill="none" aria-hidden="true">
            <path d="M5 4 C5 12, 12 12, 12 19" style="stroke:var(--ice)" stroke-width="1.6" stroke-linecap="round" />
            <path d="M12 4 C12 12, 12 12, 12 19" style="stroke:var(--teal)" stroke-width="1.6" stroke-linecap="round" />
            <path d="M19 4 C19 12, 12 12, 12 19" style="stroke:var(--violet)" stroke-width="1.6" stroke-linecap="round" />
            <circle cx="5" cy="4" r="1.8" style="fill:var(--ice)" />
            <circle cx="12" cy="4" r="1.8" style="fill:var(--teal)" />
            <circle cx="19" cy="4" r="1.8" style="fill:var(--violet)" />
            <circle cx="12" cy="20" r="2" style="fill:var(--brand-stroke)" />
        </svg>
        <span style="font-size:15px;font-weight:600;color:var(--fg-bright);">{name}</span>
    }
}

/// A sidebar for the demo shells. `route` plays the router's location.
#[component]
fn DemoNav(route: RwSignal<&'static str>) -> impl IntoView {
    let link = move |to: &'static str| {
        (
            format!("#/{to}"),
            Signal::derive(move || route.get() == to),
            Callback::new(move |_| route.set(to)),
        )
    };
    let (h_over, a_over, c_over) = link("overview");
    let (h_exec, a_exec, c_exec) = link("executions");
    let (h_wf, a_wf, c_wf) = link("workflows");
    let (h_tr, a_tr, c_tr) = link("triggers");
    let (h_gr, a_gr, c_gr) = link("graphs");
    let (h_ops, a_ops, c_ops) = link("operations");
    let (h_keys, a_keys, c_keys) = link("keys");
    view! {
        <SideNav
            label="Demo"
            footer=Box::new(|| view! {
                <div style="font-family:var(--font-mono);font-size:10px;letter-spacing:.1em;text-transform:uppercase;color:var(--faint);margin-bottom:4px;">"Connection"</div>
                <div style="font-size:12.5px;color:var(--fg-2);">"acme / prod"</div>
                <button class="cl-btn cl-btn--subtle cl-btn--xs" style="padding:0;margin-top:4px;">"Disconnect"</button>
            }.into_any())
        >
            <SideNavLink href=h_over active=a_over on_click=c_over>"Overview"</SideNavLink>
            <SideNavLink href=h_exec active=a_exec on_click=c_exec count=12usize>"Executions"</SideNavLink>
            <SideNavGroup label="Orchestration">
                <SideNavLink href=h_wf active=a_wf on_click=c_wf marker=token::ICE>"Workflows"</SideNavLink>
                <SideNavLink href=h_tr active=a_tr on_click=c_tr marker=token::VIOLET count=3usize count_color=token::BAD>"Triggers"</SideNavLink>
                <SideNavLink href=h_gr active=a_gr on_click=c_gr marker=token::TEAL>"Graphs"</SideNavLink>
            </SideNavGroup>
            <SideNavGroup label="System">
                <SideNavLink href=h_ops active=a_ops on_click=c_ops>"Operations"</SideNavLink>
                <SideNavLink href=h_keys active=a_keys on_click=c_keys>"API keys"</SideNavLink>
            </SideNavGroup>
        </SideNav>
    }
}

#[component]
pub fn FrameSections() -> impl IntoView {
    let toaster = use_toaster();

    // AppShell demo state.
    let route_a = RwSignal::new("executions");
    let route_b = RwSignal::new("workflows");

    // Modal + Drawer.
    let m_sm = RwSignal::new(false);
    let m_md = RwSignal::new(false);
    let m_lg = RwSignal::new(false);
    let m_xl = RwSignal::new(false);
    let m_locked = RwSignal::new(false);
    let drawer = RwSignal::new(false);
    let form_name = RwSignal::new(String::from("nightly-ingest"));

    // ConfirmDialog.
    let del_open = RwSignal::new(false);
    let del_busy = RwSignal::new(false);
    let arch_open = RwSignal::new(false);

    // Tabs.
    let tab = RwSignal::new(String::from("overview"));
    let route_tab = RwSignal::new(String::from("history"));
    let segment = RwSignal::new(String::from("Live"));

    // Card.
    let picked = RwSignal::new(1usize);

    let modal_footer = |open: RwSignal<bool>| -> ChildrenFn {
        std::sync::Arc::new(move || {
            view! {
                <Button variant="default" on_click=Callback::new(move |_| open.set(false))>"Cancel"</Button>
                <Button on_click=Callback::new(move |_| open.set(false))>"Save"</Button>
            }
            .into_any()
        })
    };

    view! {
        // ---- AppShell + SideNav ----
        <Section id="appshell" title="AppShell · SideNav" caption="header, brand, sidebar with groups, active link (aria-current), count badges, footer — below 768px the sidebar is a drawer behind the menu button">
            <Stack>
                <AppShell
                    contained=true
                    brand=std::sync::Arc::new(|| view! { <DemoBrand name="cloacina" /> }.into_any())
                    navbar=Box::new(move || view! { <DemoNav route=route_a /> }.into_any())
                >
                    <PageHeader title="Executions" sub="brand in the sidebar · no header" />
                    <Text dimmed=true size="sm">{move || format!("Active link: {}", route_a.get())}</Text>
                </AppShell>
                <AppShell
                    contained=true
                    brand=std::sync::Arc::new(|| view! { <DemoBrand name="kairos" /> }.into_any())
                    header=Box::new(|| view! {
                        <Group justify="between">
                            <Text dimmed=true size="sm">"colliery · tenant"</Text>
                            <Group gap="sm">
                                <Pill color=token::VIOLET>"admin"</Pill>
                                <ThemeToggle />
                            </Group>
                        </Group>
                    }.into_any())
                    navbar=Box::new(move || view! { <DemoNav route=route_b /> }.into_any())
                >
                    <PageHeader title="Workflows" sub="brand + content in the header" />
                    <Text dimmed=true size="sm">{move || format!("Active link: {}", route_b.get())}</Text>
                </AppShell>
            </Stack>
        </Section>

        // ---- PageHeader ----
        <Section id="page-header" title="PageHeader" caption="back link · <h1> title · sub line · meta pills · actions — the v1 props (title, sub, right) still work">
            <div class="gallery__card">
                <Stack>
                    <PageHeader
                        title="nightly-ingest"
                        sub="exec_7f3a01 · started 12:02:11"
                        back_href="#page-header"
                        back_label="Executions"
                        meta=Box::new(|| view! {
                            <StatusBadge status="running" />
                            <Pill color=token::TEAL>"when_all"</Pill>
                            <Pill color=token::ICE>"3 inputs"</Pill>
                        }.into_any())
                        actions=Box::new(|| view! {
                            <Button variant="default">"Re-run"</Button>
                            <Button bad=true variant="light">"Cancel run"</Button>
                        }.into_any())
                    />
                    <Divider />
                    <PageHeader
                        title="Boards"
                        sub="v1 usage: title + sub + right"
                        right=Box::new(|| view! { <Button size="xs">"New board"</Button> }.into_any())
                    />
                </Stack>
            </div>
        </Section>

        // ---- Modal + Drawer ----
        <Section id="modal" title="Modal · Drawer" caption="sizes sm 400 · md 520 · lg 760 · xl 1060 — footer slot — Escape closes — focus moves in, stays in, and goes back to the opener">
            <Group wrap=true gap="sm">
                <Button variant="default" on_click=Callback::new(move |_| m_sm.set(true))>"Small"</Button>
                <Button variant="default" on_click=Callback::new(move |_| m_md.set(true))>"Medium"</Button>
                <Button variant="default" on_click=Callback::new(move |_| m_lg.set(true))>"Large"</Button>
                <Button variant="default" on_click=Callback::new(move |_| m_xl.set(true))>"Extra large"</Button>
                <Button variant="default" on_click=Callback::new(move |_| m_locked.set(true))>"No scrim close"</Button>
                <Button variant="light" on_click=Callback::new(move |_| drawer.set(true))>"Open drawer"</Button>
            </Group>
        </Section>
        <Modal open=m_sm title="Rename schedule" size="sm" footer=modal_footer(m_sm)>
            <TextInput label="Name" value=form_name />
        </Modal>
        <Modal open=m_md title="Edit trigger" footer=modal_footer(m_md)>
            <Stack>
                <TextInput label="Name" value=form_name />
                <Text dimmed=true size="sm">"The default size, 520px."</Text>
            </Stack>
        </Modal>
        <Modal open=m_lg title="Compare versions" size="lg" footer=modal_footer(m_lg)>
            <SimpleGrid cols=2>
                <Panel title="v3"><Text mono=true size="sm">"cron: 0 2 * * *"</Text></Panel>
                <Panel title="v4"><Text mono=true size="sm">"cron: 30 1 * * *"</Text></Panel>
            </SimpleGrid>
        </Modal>
        <Modal open=m_xl title="Merge documents" size="xl" footer=modal_footer(m_xl)>
            <SimpleGrid cols=3>
                <Panel title="Ours"><Text size="sm">"Left side of the merge."</Text></Panel>
                <Panel title="Base"><Text size="sm">"Common ancestor."</Text></Panel>
                <Panel title="Theirs"><Text size="sm">"Right side of the merge."</Text></Panel>
            </SimpleGrid>
        </Modal>
        <Modal open=m_locked title="Scrim click is off" close_on_scrim=false>
            <Stack>
                <Text size="sm">"A click on the dim area does not close this dialog. Escape and × still do."</Text>
                <Group justify="end">
                    <Button on_click=Callback::new(move |_| m_locked.set(false))>"Done"</Button>
                </Group>
            </Stack>
        </Modal>
        <Drawer
            open=drawer
            title="agent-7f3a · us-east-1"
            footer=std::sync::Arc::new(move || view! {
                <Button variant="default" on_click=Callback::new(move |_| drawer.set(false))>"Close"</Button>
                <Button on_click=Callback::new(move |_| { toaster.info("Diagnostic requested"); })>"Run diagnostic"</Button>
            }.into_any())
        >
            <Stack>
                <Group gap="sm"><StatusBadge status="running" /><Pill color=token::OK>"healthy"</Pill></Group>
                <Text size="sm">"A slide-over for a detail view. Same keyboard rules as Modal."</Text>
                <Table mono=true>
                    <tbody>
                        <tr><td>"cluster"</td><td>"prod-east"</td></tr>
                        <tr><td>"heartbeat"</td><td>"4s ago"</td></tr>
                        <tr><td>"version"</td><td>"0.9.2"</td></tr>
                    </tbody>
                </Table>
            </Stack>
        </Drawer>

        // ---- ConfirmDialog ----
        <Section id="confirm" title="ConfirmDialog" caption="consequence text · a notice before the list · list of what changes · type the name to confirm · danger button · busy state">
            <Group wrap=true gap="sm">
                <Button bad=true on_click=Callback::new(move |_| del_open.set(true))>"Delete workflow…"</Button>
                <Button variant="default" on_click=Callback::new(move |_| arch_open.set(true))>"Archive board…"</Button>
            </Group>
        </Section>
        <ConfirmDialog
            open=del_open
            title="Delete workflow?"
            message="This deletes the workflow, its schedule and its run history. You cannot undo this."
            impacts=vec!["2 schedules".to_string(), "148 runs".to_string(), "1 trigger (orders-hook)".to_string()]
            impacts_label="This also deletes:"
            confirm_text="nightly-ingest"
            confirm_label="Delete workflow"
            busy=del_busy
            on_confirm=Callback::new(move |_| {
                del_busy.set(true);
                set_timeout(move || {
                    del_busy.set(false);
                    del_open.set(false);
                    toaster.success("Workflow nightly-ingest deleted");
                }, Duration::from_millis(1200));
            })
        />
        <ConfirmDialog
            open=arch_open
            title="Archive COLLIERY-I-0232?"
            message="Archiving moves the item and its children out of the board. You can restore them later."
            impacts=vec!["COLLIERY-T-1830".to_string(), "COLLIERY-T-1831".to_string(), "COLLIERY-T-1832".to_string()]
            notice=std::sync::Arc::new(|| view! {
                <Banner>"Two of these items have open blockers."</Banner>
            }.into_any())
            confirm_label="Archive"
            danger=false
            on_confirm=Callback::new(move |_| {
                arch_open.set(false);
                toaster.toast(ToastKind::Warning, "3 items archived");
            })
        />

        // ---- Toast ----
        <Section id="toast" title="Toast" caption="use_toaster() from anywhere — bottom-right stack — auto-dismiss after 5s, paused on hover — click to dismiss — role=status, role=alert for errors">
            <Group wrap=true gap="sm">
                <Button variant="default" on_click=Callback::new(move |_| { toaster.info("Schedule saved as a draft"); })>"Info"</Button>
                <Button variant="default" on_click=Callback::new(move |_| { toaster.success("Workflow deployed"); })>"Success"</Button>
                <Button variant="default" on_click=Callback::new(move |_| { toaster.warning("2 inputs are stale"); })>"Warning"</Button>
                <Button variant="default" on_click=Callback::new(move |_| { toaster.error("The server did not answer"); })>"Error"</Button>
                <Button variant="subtle" on_click=Callback::new(move |_| { toaster.toast_for(ToastKind::Info, "This one stays until you dismiss it", 0); })>"Sticky"</Button>
            </Group>
        </Section>

        // ---- Tabs ----
        <Section id="tabs" title="Tabs" caption="underline tab list — controlled by a signal (buttons) or by links (routes) — arrow keys, Home, End — compare SegmentedControl, a boxed choice of values">
            <div class="gallery__card">
                <Stack>
                    <Tabs
                        label="Workflow"
                        value=tab
                        tabs=vec![
                            TabItem::new("overview", "Overview"),
                            TabItem::new("runs", "Runs").count(12),
                            TabItem::new("settings", "Settings"),
                            TabItem::new("audit", "Audit").disabled(true),
                        ]
                    >
                        <TabPanel value="overview"><Text size="sm">"Overview panel. Use the arrow keys on the tab list."</Text></TabPanel>
                        <TabPanel value="runs"><Text size="sm">"Runs panel: 12 runs."</Text></TabPanel>
                        <TabPanel value="settings"><Text size="sm">"Settings panel."</Text></TabPanel>
                        <TabPanel value="audit"><Text size="sm">"Audit panel."</Text></TabPanel>
                    </Tabs>
                    <Divider />
                    <Text dimmed=true size="xs">"Route tabs (links; the page under them is the router outlet):"</Text>
                    <Tabs
                        label="Execution views"
                        value=route_tab
                        tabs=vec![
                            TabItem::new("history", "History").href("#tabs"),
                            TabItem::new("current", "Current run").href("#tabs"),
                            TabItem::new("events", "Events").href("#tabs").count(48),
                        ]
                    />
                    <Divider />
                    <Group gap="sm">
                        <Text dimmed=true size="xs">"SegmentedControl, for contrast:"</Text>
                        <SegmentedControl value=segment options=vec!["Live".into(), "Paused".into(), "Replay".into()] />
                    </Group>
                </Stack>
            </div>
        </Section>

        // ---- Card ----
        <Section id="card" title="Card" caption="a panel a person can click or open with the keyboard — href (a link) or on_click (a button) — hover, focus and selected states">
            <SimpleGrid cols=3>
                <Card href="#card" title="Platform board">
                    <Text dimmed=true size="sm">"A link card. 14 items · 3 in progress."</Text>
                </Card>
                <Card
                    title="orders-flow"
                    on_click=Callback::new(move |_| picked.set(1))
                    selected=Signal::derive(move || picked.get() == 1)
                >
                    <Text dimmed=true size="sm">"A button card. Click or press Enter to select."</Text>
                </Card>
                <Card title="Static card">
                    <Text dimmed=true size="sm">"No href, no on_click: a plain panel."</Text>
                </Card>
            </SimpleGrid>
        </Section>
    }
}
