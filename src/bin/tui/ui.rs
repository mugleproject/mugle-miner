// Copyright 2020 The Mugle Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Basic TUI to better output the overall system status and status
//! of various subsystems

use std::sync::{mpsc, Arc, RwLock};
use std::{self, thread};
use time;

use cursive::direction::Orientation;
use cursive::theme::BaseColor::*;
use cursive::theme::Color::*;
use cursive::theme::PaletteColor::*;
use cursive::theme::{BaseColor, BorderStyle, Color, Theme};
use cursive::traits::*;
use cursive::utils::markup::StyledString;
use cursive::views::{BoxedView, LinearLayout, Panel, StackView, TextView};
use cursive::Cursive;

use tui::constants::*;
use tui::types::*;
use tui::{menu, mining, version};

use stats;

use built_info;

/// Main UI
pub struct UI {
	cursive: Cursive,
	ui_rx: mpsc::Receiver<UIMessage>,
	ui_tx: mpsc::Sender<UIMessage>,
	controller_tx: mpsc::Sender<ControllerMessage>,
}

fn modify_theme(theme: &mut Theme) {
	theme.shadow = false;
	theme.borders = BorderStyle::Simple;
	theme.palette[Background] = Dark(Black);
	theme.palette[Shadow] = Dark(Black);
	theme.palette[View] = Dark(Black);
	theme.palette[Primary] = Dark(White);
	theme.palette[Highlight] = Dark(Cyan);
	theme.palette[HighlightInactive] = Dark(Blue);
	// also secondary, tertiary, TitlePrimary, TitleSecondary
}

impl UI {
	/// Create a new UI
	pub fn new(controller_tx: mpsc::Sender<ControllerMessage>) -> UI {
		let (ui_tx, ui_rx) = mpsc::channel::<UIMessage>();
		let mut mugle_ui = UI {
			cursive: Cursive::default(),
			ui_tx,
			ui_rx,
			controller_tx,
		};

		// Create UI objects, etc
		let mining_view = mining::TUIMiningView::create();
		let version_view = version::TUIVersionView::create();

		let main_menu = menu::create();

		let root_stack = StackView::new()
			.layer(version_view)
			.layer(mining_view)
			.with_name(ROOT_STACK);

		let mut title_string = StyledString::new();
		title_string.append(StyledString::styled(
			format!("Mugle Miner Version {}", built_info::PKG_VERSION),
			Color::Dark(BaseColor::Yellow),
		));

		let main_layer = LinearLayout::new(Orientation::Vertical)
			.child(Panel::new(TextView::new(title_string)))
			.child(
				LinearLayout::new(Orientation::Horizontal)
					.child(Panel::new(BoxedView::new(main_menu)))
					.child(Panel::new(root_stack)),
			);

		//set theme
		let mut theme = mugle_ui.cursive.current_theme().clone();
		modify_theme(&mut theme);
		mugle_ui.cursive.set_theme(theme);
		mugle_ui.cursive.add_layer(main_layer);

		// Configure a callback (shutdown, for the first test)
		let controller_tx_clone = mugle_ui.controller_tx.clone();
		mugle_ui.cursive.add_global_callback('q', move |_| {
			controller_tx_clone
				.send(ControllerMessage::Shutdown)
				.unwrap();
		});
		mugle_ui.cursive.set_fps(4);
		mugle_ui
	}

	/// Step the UI by calling into Cursive's step function, then
	/// processing any UI messages
	pub fn step(&mut self) -> bool {
		if !self.cursive.is_running() {
			return false;
		}

		// Process any pending UI messages
		while let Some(message) = self.ui_rx.try_iter().next() {
			match message {
				UIMessage::UpdateStatus(update) => {
					mining::TUIMiningView::update(&mut self.cursive, update.clone());
					version::TUIVersionView::update(&mut self.cursive, update.clone());
				}
			}
		}

		// Step the UI
		self.cursive.step();
		true
	}

	/// Stop the UI
	pub fn stop(&mut self) {
		self.cursive.quit();
	}
}

/// Controller message

pub struct Controller {
	rx: mpsc::Receiver<ControllerMessage>,
	ui: UI,
}

/// Controller Message
pub enum ControllerMessage {
	/// Shutdown
	Shutdown,
}

impl Controller {
	/// Create a new controller
	pub fn new() -> Result<Controller, String> {
		let (tx, rx) = mpsc::channel::<ControllerMessage>();
		Ok(Controller {
			rx,
			ui: UI::new(tx),
		})
	}
	/// Run the controller
	pub fn run(&mut self, stats: Arc<RwLock<stats::Stats>>) {
		let stat_update_interval = 1;
		let mut next_stat_update = time::get_time().sec + stat_update_interval;
		while self.ui.step() {
			if let Some(message) = self.rx.try_iter().next() {
				match message {
					ControllerMessage::Shutdown => {
						self.ui.stop();
						return;
					}
				}
			}
			if time::get_time().sec > next_stat_update {
				self.ui
					.ui_tx
					.send(UIMessage::UpdateStatus(stats.clone()))
					.unwrap();
				next_stat_update = time::get_time().sec + stat_update_interval;
			}
			thread::sleep(std::time::Duration::from_millis(100));
		}
	}
}
