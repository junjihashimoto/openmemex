use crate::add_note::*;
use crate::api::*;
use crate::app_router::*;
use crate::cards::*;
use crate::tags::*;
use std::collections::HashSet;
use yew::services::fetch::{FetchService, FetchTask, Request, Response};
use yew::{
    format::{Json, Nothing},
    prelude::*,
    utils::host,
};

use yew_router::prelude::*;

pub type Link = RouterAnchor<AppRoute>;

#[derive(Debug)]
pub struct App {
    cache_task: Option<FetchTask>,
    tag_task: Option<FetchTask>,
    entries: Option<Vec<Cache>>,
    tags: Option<Vec<String>>,
    selected_tags: HashSet<String>,
    link: ComponentLink<Self>,
    error: Option<String>,
    query: String,
}

#[derive(Debug)]
pub enum AppMsg {
    GetEntries,
    ReceiveEntries(Result<Vec<Cache>, anyhow::Error>),
    ReceiveTags(Result<Vec<String>, anyhow::Error>),
    KeyDown,
    TagClick(Option<String>),
    SortByDate,
    SortByUrl,
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
                <a class="navbar-brand" href="#"> { "note2self" } </a>
                <div class="collapse navbar-collapse" id="navbarNav">
                    <ul class="navbar-nav">
                        <li class="nav-item active">
                            <Link route=AppRoute::Gallery><div class="nav-link">{ "Gallery" }</div></Link>
                        </li>
                        <li class="nav-item" accesskey="a">
                            <Link route=AppRoute::AddNote><div class="nav-link">{ "Create" }</div></Link>
                        </li>
                        <li class="nav-item">
                            <a class="nav-link" href="#">{ "Detail (TODO)" }</a>
                        </li>
                        <li class="nav-item">
                            <a class="nav-link" href="#">{ "Queue (TODO)" }</a>
                        </li>
                        /*
                        <li class="nav-item">
                            <a class="nav-link" href="#" accesskey="q">{ "Queue" }</a>
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
        Self {
            cache_task: None,
            tag_task: None,
            entries: None,
            tags: None,
            selected_tags: HashSet::new(),
            link,
            error: None,
            query: format!("http://{}/all/cache", server).to_string(),
        }
    }

    fn change(&mut self, _props: Self::Properties) -> bool {
        false
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        let server = host().unwrap();
        log::info!("host is {:?}", server);
        match msg {
            AppMsg::GetEntries => {
                // define request
                log::info!("submitting cache request");
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
                let request = Request::get(format!("http://{}/all/tags", server))
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
                        // log::info!("Update: {:#?}", result);
                        self.entries = Some(result);
                    }
                    Err(error) => {
                        log::info!("cache receive error, error is:");
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
                false
            }
            AppMsg::KeyDown => {
                log::info!("keydown event");
                false
            }
            AppMsg::TagClick(tag) => {
                log::info!("tag click event");
                log::info!("{:?}", tag);
                let query = match tag {
                    Some(tag_name) => {
                        format!("http://{}/all/cache?sort=time&tag={}", server, tag_name)
                    }
                    None => {
                        format!("http://{}/all/cache?sort=time", server)
                    }
                };
                log::info!("Query is: {:?}", &query);
                self.query = query.clone(); // TODO - make queryparams compose
                self.link.send_message(AppMsg::GetEntries);
                false
            }
            AppMsg::SortByDate => {
                log::info!("sort date");
                self.query = format!("http://{}/all/cache?sort=time", server).to_string();
                self.link.send_message(AppMsg::GetEntries);
                false
            }
            AppMsg::SortByUrl => {
                log::info!("sort url");
                self.query = format!("http://{}/all/cache?sort=url", server).to_string();
                self.link.send_message(AppMsg::GetEntries);
                false
            }
        }
    }

    fn view(&self) -> Html {
        let empty_vec = &[].to_vec();
        let exist_tags = self.tags.as_ref().unwrap_or(empty_vec);
        let callback = self.link.callback(move |tag| AppMsg::TagClick(tag));

        let gallery = html! {
            <div>
                <div>
                    <input type="text" class="search-input" placeholder="TODO DPR Search" accesskey="/" />
                </div>
                <center>
                <div class="btn-group">
                    <button class="sort-button" onclick=self.link.callback(|m| { AppMsg::SortByDate
                        })>{"Sort by Date"}</button>
                    <button class="sort-button" onclick=self.link.callback(|m| { AppMsg::SortByUrl
                        })>{"Sort by Url"}</button>
                </div>
                </center>
                <p/>
                <div class="twocol">
                    <Cards entries=self.entries.clone()/>
                    <Tags tags=exist_tags tag_click_callback=callback/>
                </div>
            </div>
        };

        let render = Router::render(move |switch: AppRoute| match switch {
            AppRoute::Gallery => gallery.clone(),
            AppRoute::AddNote => html! { <AddNote/> },
        });

        html! {
            <div class="main-outer" onkeydown={ self.link.callback(move |e: KeyboardEvent|
                { e.stop_propagation(); AppMsg::KeyDown })}>
                { self.view_navbar() }
                <div class="main-inner">
                    <div class="main-top">
                        <h1 class="big-title"> { "note2self" } </h1>
                        <hr/>
                        <Router<AppRoute, ()> render=render/>
                    </div>
                </div>
            </div>
        }
    }
}
