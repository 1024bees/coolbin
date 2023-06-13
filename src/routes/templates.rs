use std::collections::HashMap;

use axum::{
    extract::{Path, Query},
    response::Html,
    Json,
};

use askama::Template;
use axum_sessions::extractors::WritableSession;

#[derive(Template)]
#[template(path = "chart_landing_template.html")]
pub struct ChartDemo {
    selectors: DropwdownTemplate,
}

#[derive(Template)]
#[template(path = "select_fragment.html")]
pub struct DropwdownTemplate {
    name: String,
    init_options: &'static str,
    selectors: Vec<Selector>,
}

#[derive(Template)]
#[template(path = "options_fragment.html")]
pub struct Options {
    options: Vec<String>,
}

struct Selector {
    name: &'static str,
    selected_option: String,
    htmx_options: String,
}

impl Selector {
    fn to_default_htmx_options(&self) -> String {
        format!(
            r##" hx-get=/htmx_demo/selector/{} hx-target=#{} hx-trigger=htmx:afterSettle,change hx-indicator=.htmx-indicator "##,
            self.name, self.name
        )
    }
}

impl DropwdownTemplate {
    fn some_default() -> Self {
        let sel3 = Selector {
            name: "git_sha",
            selected_option: "".into(),
        
            htmx_options: r##" hx-get=/htmx_demo/graph_data    hx-ext=chartjs  hx-trigger=change hx-swap=none "##
                .to_string(),
        };

        let sel2 = Selector {
            name: "branch",
            selected_option: "".into(),

            htmx_options: sel3.to_default_htmx_options(),
        };

        let sel1 = Selector {
            name: "repo",
            selected_option: "".into(),

            htmx_options: sel2.to_default_htmx_options(),
        };

        DropwdownTemplate {
            name: "demo".into(),
            init_options: r##"hx-get=/htmx_demo/selector/repo hx-target=#repo hx-trigger=load,change "##,
            selectors: vec![sel1, sel2, sel3],
        }
    }
}

pub async fn test_graph() -> Html<String> {
    let chart_demo = ChartDemo {
        selectors: DropwdownTemplate::some_default(),
    };
    Html(chart_demo.render().unwrap())
}

#[derive(serde::Serialize)]
pub struct ChartData {
    labels: Vec<String>,
    datasets: Vec<ChartJsDataset>,
}
#[derive(serde::Serialize)]
pub struct ChartJsDataset {
    label: String,
    #[serde(rename = "backgroundColor")]
    background_color: String,
    data: Vec<f64>,
}

pub async fn generate_random_chart(
    query_params: Query<HashMap<String, String>>,
) -> Json<ChartData> {
    tracing::info!("Getting chart data!");

    let first_ds = ChartJsDataset {
        label: "Demo".into(),
        background_color: "rgba(255,165,0,0.5)".into(),
        data: (0..5).map(|val| val as f64).collect(),
    };
    let second_ds = ChartJsDataset {
        label: "Demo2".into(),
        background_color: "rgba(20,15,255,0.5)".into(),
        data: (0..5).rev().map(|val| val as f64).collect(),
    };

    let labels = (0..5).map(|val| format!("head~{}", val)).rev().collect();

    Json(ChartData {
        labels,
        datasets: vec![first_ds, second_ds],
    })
}



#[tracing::instrument(name = "getting selectors")]
pub async fn selector_demo_path(
    Path(name): Path<String>,
    Query(query_params): Query<HashMap<String, String>>,
    mut session: WritableSession,
) -> Html<String> {
    let mut state : HashMap<String,String>= session.get("page-state").unwrap_or(HashMap::new());
    
    state.extend(query_params.clone().into_iter());
    
    tracing::info!("State params are: {:?} is {:?}",query_params, state);
session.insert("page-state", state).unwrap();

    let options = (0..5).map(|val| format!("{}_{}", name, val)).collect();
    let opts = Options { options };
    tracing::info!("Rendering path, ");
    Html(opts.render().unwrap())
}
