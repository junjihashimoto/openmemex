use crate::add_note::*;
use crate::api::*;
use crate::app_router::*;
use crate::cards::*;
use crate::detail::*;
use crate::space::*;
use crate::queue::*;
use crate::settings::*;
use crate::timeline::*;
use crate::tags::*;
use std::collections::HashSet;
use yew::services::fetch::{FetchService, FetchTask, Request, Response};
use yew::{
    format::{Json, Nothing},
    prelude::*,
    utils::host,
};
use urlencoding::encode;
use yew_router::prelude::*;
use chrono::*;

pub type Link = RouterAnchor<AppRoute>;

#[derive(Debug)]
pub struct App {
    cache_task: Option<FetchTask>,
    tag_task: Option<FetchTask>,
    entries: Option<Vec<Cache>>,
    selected_entry: Option<Cache>,
    tags: Option<Vec<String>>,
    selected_tags: HashSet<String>,
    link: ComponentLink<Self>,
    error: Option<String>,
    default_query: String,
    query: String,
    search_query: String,
}

#[derive(Debug)]
pub enum AppMsg {
    GetEntries,
    ReceiveEntries(Result<Vec<Cache>, anyhow::Error>),
    ReceiveTags(Result<Vec<String>, anyhow::Error>),
    KeyDown,
    // callback events
    CardClick(Option<Cache>),
    TagClick(Option<String>),
    TimelineEvt(Option<(NaiveDateTime, NaiveDateTime)>),

    //
    SortByDate,
    SortByUrl,
    SearchEdit(String),
    SearchKeyDown(KeyboardEvent),
    SearchSubmit,
}

impl App {
    fn view_navbar(&self) -> Html {
        html! {
            <nav class="navbar navbar-expand-lg navbar-light bg-light">

                /*
                <div class="navbar-header">
                    <button type="button" class="navbar-toggle" data-toggle="collapse" data-target="#navbarNav">
                        <span class="icon-bar"></span>
                        <span class="icon-bar"></span>
                        <span class="icon-bar"></span>
                    </button>
                </div>
                */

                <a class="navbar-brand" href="/frontend/index.html"> <span style="color:#bb7b52">{"Open"}</span><span style="color:#000000">{"Memex"}</span> </a>
                <div class="collapse navbar-collapse" id="navbarNav">
                    <ul class="navbar-nav">
                        <li class="nav-item active">
                            <Link route=AppRoute::Gallery><div class="nav-link">{ "Gallery" }</div></Link>
                        </li>
                        <li class="nav-item" accesskey="a">
                            <Link route=AppRoute::AddNote><div class="nav-link">{ "Create" }</div></Link>
                        </li>
                        <li class="nav-item" accesskey="d">
                            <Link route=AppRoute::Detail><div class="nav-link">{ "Detail" }</div></Link>
                        </li>
                        /*
                        <li class="nav-item" accesskey="s">
                            <Link route=AppRoute::Space><div class="nav-link">{ "Space" }</div></Link>
                        </li>
                        <li class="nav-item" accesskey="q">
                            <Link route=AppRoute::Queue><div class="nav-link">{ "Queue" }</div></Link>
                        </li>
                        <li class="nav-item" accesskey=",">
                            <Link route=AppRoute::Settings><div class="nav-link">{ "Settings" }</div></Link>
                        </li>
                        */
                    </ul>
                </div>
            </nav>
        }
    }
}

impl Component for App {
    type Message = AppMsg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let server = host().unwrap();
        log::info!("Creating component");
        let cb = link.callback_once(|_: String| AppMsg::GetEntries);
        cb.emit("".to_string()); // TODO - what's the right way to handle a message without parameters
        log::info!("sent message");
        // let kb_cb = link.callback(Msg::KeyDown);
        let default_query = format!("http://{}/all/cache?limit=150", server).to_string();
        Self {
            cache_task: None,
            tag_task: None,
            entries: None,
            tags: None,
            selected_entry: None,
            selected_tags: HashSet::new(),
            link,
            error: None,
            default_query: default_query.clone(),
            query: default_query.clone(),
            search_query: String::from(""),
        }
    }

    fn change(&mut self, _props: Self::Properties) -> bool {
        true
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        let server = host().unwrap();
        log::info!("host is {:?}", server);
        match msg {
            AppMsg::GetEntries => {
                // define request
                log::info!("submitting cache request: {:?}", self.query);
                let request = Request::get(&self.query)
                    .body(Nothing)
                    .expect("Could not build request.");
                // define callback
                let callback = self.link.callback_once(
                    |response: Response<Json<Result<Vec<Cache>, anyhow::Error>>>| {
                        let Json(data) = response.into_body();
                        AppMsg::ReceiveEntries(data)
                    },
                );
                // task
                let task = FetchService::fetch(request, callback).expect("failed to start request");
                self.cache_task = Some(task);
                // define request
                log::info!("submitting tag request");
                let request = Request::get(format!("http://{}/all/tags?min=10", server))
                    .body(Nothing)
                    .expect("Could not build request.");
                // define callback
                let callback = self.link.callback_once(
                    |response: Response<Json<Result<Vec<String>, anyhow::Error>>>| {
                        let Json(data) = response.into_body();
                        AppMsg::ReceiveTags(data)
                    },
                );
                // task
                let task = FetchService::fetch(request, callback).expect("failed to start request");
                self.tag_task = Some(task);
                true // redraw page
            }
            AppMsg::ReceiveEntries(response) => {
                match response {
                    Ok(result) => {
                        self.entries = Some(result);
                    }
                    Err(error) => {
                        log::info!("cache receive error:");
                        log::info!("{}", &error.to_string());
                        self.error = Some(error.to_string());
                    }
                }
                self.cache_task = None;
                true
            }
            AppMsg::ReceiveTags(response) => {
                match response {
                    Ok(result) => {
                        self.tags = Some(result);
                    }
                    Err(error) => {
                        log::info!("tag receive error, error is:");
                        log::info!("{}", &error.to_string());
                        self.error = Some(error.to_string());
                    }
                }
                self.tag_task = None;
                true
            }
            AppMsg::KeyDown => {
                log::info!("keydown event");
                false
            }
            AppMsg::CardClick(entry) => {
                self.selected_entry = entry;
                log::info!("selected entry is {:?}", self.selected_entry);
                true
            }
            AppMsg::TagClick(tag) => {
                log::info!("tag click event");
                log::info!("{:?}", tag);
                self.query = match tag {
                    Some(tag_name) => {
                        format!("http://{}/all/cache?sort=time&tag={}&limit=150", server, tag_name)
                    }
                    None => format!("http://{}/all/cache?sort=time&limit=150", server),
                };
                log::info!("Query is: {:?}", &self.query);
                // self.query = query.clone(); // TODO - make queryparams compose
                self.link.send_message(AppMsg::GetEntries);
                false
            }
            AppMsg::TimelineEvt(evt) => {
                log::info!("Timeline event");
                self.query = match evt {
                    Some((dt_min, dt_max)) => {
                        format!("http://{}/all/cache?sort=time&startDate={}&endDate={}&limit=150", server,
                            dt_min.format("%Y-%m-%d").to_string(), 
                            dt_max.format("%Y-%m-%d").to_string())
                    }
                    None => format!("http://{}/all/cache?sort=time&limit=150", server),
                };
                log::info!("Query is: {:?}", &self.query);
                self.link.send_message(AppMsg::GetEntries);
                false
            }
            AppMsg::SortByDate => {
                log::info!("sort date");
                self.query = format!("http://{}/all/cache?sort=time&limit=150", server).to_string();
                self.link.send_message(AppMsg::GetEntries);
                true
            }
            AppMsg::SortByUrl => {
                log::info!("sort url");
                self.query = format!("http://{}/all/cache?sort=url&limit=150", server).to_string();
                self.link.send_message(AppMsg::GetEntries);
                true
            }
            AppMsg::SearchKeyDown(keypress) => {
                log::info!("search keydown {:?}", keypress.key());
                if keypress.key() == "Enter" {
                    self.link.send_message(AppMsg::SearchSubmit);
                }
                false
            }
            AppMsg::SearchEdit(query) => {
                self.search_query = query;
                false
            }
            AppMsg::SearchSubmit => {
                self.query = format!("http://{}/search/{}", server, encode(self.search_query.trim()).to_string());
                log::info!("Query: {}", &self.query);
                self.link.send_message(AppMsg::GetEntries);
                false
            }
        }
    }

    fn view(&self) -> Html {
        let empty_vec = &[].to_vec();
        let exist_tags = self.tags.as_ref().unwrap_or(empty_vec);
        let card_callback = self.link.callback(move |card| AppMsg::CardClick(card));
        let tag_callback = self.link.callback(move |tag| AppMsg::TagClick(tag));
        let timeline_callback = self.link.callback(move |dt| AppMsg::TimelineEvt(dt));

        let button_class = "sort-button shadow-sm p-3 mb-5 bg-white rounded";

        let gallery = html! {
            <div>
                /*
                    <button class=button_class onclick=self.link.callback(|m| { 
                        AppMsg::SortByDate
                        })> {"▼ Date"}</button>
                */
                <input type="text" class="search-input shadow-sm p-3 mb-5 bg-white rounded" placeholder="Search" accesskey="/" 
                oninput = { self.link.callback(move |e: InputData| AppMsg::SearchEdit(e.value)) }
                onkeydown = { self.link.callback(move |e: KeyboardEvent| AppMsg::SearchKeyDown(e)) }
                />
                <Timeline timeline_callback = timeline_callback/>
                <p/>
                <div class="twocol">
                    <Cards entries=self.entries.clone() card_click_callback=card_callback/>
                    <div>
                        <Tags tags=exist_tags.clone() tag_click_callback=tag_callback/>
                        //<p/>
                        //<input type="checkbox" id="hidecompleted" name="hidecompleted"/>
                        //<label style="height:10%; margin-left: 10px"> {"Hide Completed"} </label>
                    </div>
                </div>
            </div>
        };

        let entry = self.selected_entry.clone();

        log::info!("switch with entry as {:?}", &entry);
        let render = Router::render(move |switch: AppRoute| match switch {
            AppRoute::Gallery => gallery.clone(),
            AppRoute::AddNote => html! { <AddNote/> },
            AppRoute::Detail => html! { <Detail entry=entry.clone() /> },
            AppRoute::Space => html! { <Space /> },
            AppRoute::Queue => html! { <Queue /> },
            AppRoute::Settings => html! { <Settings/> },
        });

        html! {
            <div class="main-outer" onkeydown={ self.link.callback(move |e: KeyboardEvent|
                { e.stop_propagation(); AppMsg::KeyDown })}>
                { self.view_navbar() }
                <div class="main-inner">
                    <div class="main-top">
                    /*
                        <a href="/frontend/index.html" style="text-decoration: none">
                        <h1 class="big-title"> <span style="color:#bb7b52">{"Open"}</span><span style="color:#000000">{"Memex"}</span></h1>
                        </a>
                        <hr/>
                    */
                        <Router<AppRoute, ()> render=render/>
                    </div>
                </div>
            </div>
        }
    }
}
