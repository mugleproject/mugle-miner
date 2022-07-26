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

//! Version and build info

use std::sync::{Arc, RwLock};

use cursive::direction::Orientation;
use cursive::traits::*;
use cursive::view::View;
use cursive::views::{LinearLayout, ResizedView, TextView};
use cursive::Cursive;

use tui::constants::*;
use tui::types::*;

use info_strings;
use stats::Stats;

/// Version view
pub struct TUIVersionView;

impl TUIStatusListener for TUIVersionView {
	/// Create basic status view
	fn create() -> Box<dyn View> {
		let (basic_info, detailed_info, _) = info_strings();
		let basic_status_view = ResizedView::with_full_screen(
			LinearLayout::new(Orientation::Vertical)
				.child(TextView::new(basic_info))
				.child(TextView::new(" "))
				.child(TextView::new(detailed_info)),
		);
		Box::new(basic_status_view.with_name(VIEW_VERSION))
	}

	/// update
	fn update(_c: &mut Cursive, _stats: Arc<RwLock<Stats>>) {}
}
