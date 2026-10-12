//! Gallery section for the controls of AURORA-T-0013 item 22: Checkbox,
//! FileInput, Slider, Combobox, Disclosure and Accordion, Skeleton, Loader.

use aurora_leptos::components::*;
use leptos::prelude::*;

use crate::Section;

#[component]
pub fn ControlsSections() -> impl IntoView {
    let seed = RwSignal::new(true);
    let all = RwSignal::new(false);
    let some = Signal::derive(move || !all.get());
    let files = RwSignal::new(String::from("none yet"));
    let volume = RwSignal::new(40.0_f64);
    let kind = RwSignal::new(String::from("tv"));
    let kinds = vec![
        ("movie".to_string(), "Movie".to_string()),
        ("tv".to_string(), "TV show".to_string()),
        ("audiobook".to_string(), "Audiobook".to_string()),
        ("podcast".to_string(), "Podcast".to_string()),
        ("music".to_string(), "Music".to_string()),
    ];
    let details_open = RwSignal::new(false);
    let loading = RwSignal::new(true);

    view! {
        <Section id="controls" title="Checkbox · FileInput · Slider · Combobox" caption="native controls where HTML has one; the field props of TextInput (label, description, error)">
            <div class="gallery__card">
                <Stack>
                    <Group gap="lg" wrap=true top=true>
                        <Checkbox checked=seed label="Seed after import" description="Keeps the torrent in the client." />
                        <Checkbox checked=all label="Select all" indeterminate=some />
                        <Checkbox checked=RwSignal::new(false) label="Disabled" disabled=true />
                    </Group>
                    <FileInput
                        label="Subtitles"
                        accept=".srt,.vtt"
                        multiple=true
                        description="SubRip or WebVTT."
                        on_change=Callback::new(move |fs: Vec<leptos::web_sys::File>| {
                            files.set(format!("{} file(s)", fs.len()));
                        })
                    />
                    <Text size="xs" dimmed=true>{move || format!("on_change: {}", files.get())}</Text>
                    <Slider value=volume label="Volume" format=Callback::new(|v: f64| format!("{v}%")) />
                    <Combobox value=kind options=kinds label="Kind" placeholder="Type to filter" description="ArrowDown opens the list." />
                    <Text size="xs" dimmed=true>{move || format!("value: {}", kind.get())}</Text>
                </Stack>
            </div>
        </Section>
        <Section id="disclosure" title="Disclosure · Accordion · Skeleton · Loader" caption="details/summary sections, one open at a time in an Accordion; placeholders while content loads">
            <div class="gallery__card">
                <Stack>
                    <Disclosure title="Advanced options" open=details_open>
                        <Text size="sm">"A Disclosure on its own, bound to a signal."</Text>
                    </Disclosure>
                    <Text size="xs" dimmed=true>{move || format!("open: {}", details_open.get())}</Text>
                    <Accordion>
                        <Disclosure title="Library" default_open=true>
                            <Text size="sm">"Opening another section closes this one."</Text>
                        </Disclosure>
                        <Disclosure title="Downloads">
                            <Text size="sm">"Clients, paths, limits."</Text>
                        </Disclosure>
                        <Disclosure title="Users">
                            <Text size="sm">"Household members and roles."</Text>
                        </Disclosure>
                    </Accordion>
                    <Group gap="sm">
                        <Switch checked=loading label="Loading" />
                        <Loader size="xs" />
                        <Loader size="sm" />
                        <Loader label="Loading runs" />
                    </Group>
                    {move || if loading.get() {
                        view! {
                            <Group top=true>
                                <Skeleton circle=true height="40px" />
                                <div style="flex:1;"><Skeleton lines=3 /></div>
                            </Group>
                        }.into_any()
                    } else {
                        view! { <Text size="sm">"The content, loaded."</Text> }.into_any()
                    }}
                </Stack>
            </div>
        </Section>
    }
}
