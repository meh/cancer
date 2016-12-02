// Copyleft (ↄ) meh. <meh@schizofreni.co> | http://meh.schizofreni.co
//
// This file is part of cancer.
//
// cancer is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// cancer is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with cancer.  If not, see <http://www.gnu.org/licenses/>.

use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use fnv::FnvHasher;

use toml;
use palette::Rgba;

use config::util::to_color;

#[derive(PartialEq, Clone, Debug)]
pub struct Color {
	table: HashMap<u8, Rgba<f64>, BuildHasherDefault<FnvHasher>>,
}

// http://www.calmar.ws/vim/256-xterm-24bit-rgb-color-chart.html
impl Default for Color {
	fn default() -> Self {
		let mut table = HashMap::default();

		// Base 16 colors.
		table.insert(0,  to_color("#000000").unwrap());
		table.insert(1,  to_color("#cd0000").unwrap());
		table.insert(2,  to_color("#00cd00").unwrap());
		table.insert(3,  to_color("#cdcd00").unwrap());
		table.insert(4,  to_color("#0000cd").unwrap());
		table.insert(5,  to_color("#cd00cd").unwrap());
		table.insert(6,  to_color("#00cdcd").unwrap());
		table.insert(7,  to_color("#e5e5e5").unwrap());
		table.insert(8,  to_color("#7f7f7f").unwrap());
		table.insert(9,  to_color("#ff0000").unwrap());
		table.insert(10, to_color("#00ff00").unwrap());
		table.insert(11, to_color("#ffff00").unwrap());
		table.insert(12, to_color("#5c5cff").unwrap());
		table.insert(13, to_color("#ff00ff").unwrap());
		table.insert(14, to_color("#00ffff").unwrap());
		table.insert(15, to_color("#ffffff").unwrap());

		// Extended 240 colors.
		table.insert(16,  to_color("#000000").unwrap());
		table.insert(17,  to_color("#00005f").unwrap());
		table.insert(18,  to_color("#000087").unwrap());
		table.insert(19,  to_color("#0000af").unwrap());
		table.insert(20,  to_color("#0000d7").unwrap());
		table.insert(21,  to_color("#0000ff").unwrap());
		table.insert(22,  to_color("#005f00").unwrap());
		table.insert(23,  to_color("#005f5f").unwrap());
		table.insert(24,  to_color("#005f87").unwrap());
		table.insert(25,  to_color("#005faf").unwrap());
		table.insert(26,  to_color("#005fd7").unwrap());
		table.insert(27,  to_color("#005fff").unwrap());
		table.insert(28,  to_color("#008700").unwrap());
		table.insert(29,  to_color("#00875f").unwrap());
		table.insert(30,  to_color("#008787").unwrap());
		table.insert(31,  to_color("#0087af").unwrap());
		table.insert(32,  to_color("#0087d7").unwrap());
		table.insert(33,  to_color("#0087ff").unwrap());
		table.insert(34,  to_color("#00af00").unwrap());
		table.insert(35,  to_color("#00af5f").unwrap());
		table.insert(36,  to_color("#00af87").unwrap());
		table.insert(37,  to_color("#00afaf").unwrap());
		table.insert(38,  to_color("#00afd7").unwrap());
		table.insert(39,  to_color("#00afff").unwrap());
		table.insert(40,  to_color("#00d700").unwrap());
		table.insert(41,  to_color("#00d75f").unwrap());
		table.insert(42,  to_color("#00d787").unwrap());
		table.insert(43,  to_color("#00d7af").unwrap());
		table.insert(44,  to_color("#00d7d7").unwrap());
		table.insert(45,  to_color("#00d7ff").unwrap());
		table.insert(46,  to_color("#00ff00").unwrap());
		table.insert(47,  to_color("#00ff5f").unwrap());
		table.insert(48,  to_color("#00ff87").unwrap());
		table.insert(49,  to_color("#00ffaf").unwrap());
		table.insert(50,  to_color("#00ffd7").unwrap());
		table.insert(51,  to_color("#00ffff").unwrap());
		table.insert(52,  to_color("#5f0000").unwrap());
		table.insert(53,  to_color("#5f005f").unwrap());
		table.insert(54,  to_color("#5f0087").unwrap());
		table.insert(55,  to_color("#5f00af").unwrap());
		table.insert(56,  to_color("#5f00d7").unwrap());
		table.insert(57,  to_color("#5f00ff").unwrap());
		table.insert(58,  to_color("#5f5f00").unwrap());
		table.insert(59,  to_color("#5f5f5f").unwrap());
		table.insert(60,  to_color("#5f5f87").unwrap());
		table.insert(61,  to_color("#5f5faf").unwrap());
		table.insert(62,  to_color("#5f5fd7").unwrap());
		table.insert(63,  to_color("#5f5fff").unwrap());
		table.insert(64,  to_color("#5f8700").unwrap());
		table.insert(65,  to_color("#5f875f").unwrap());
		table.insert(66,  to_color("#5f8787").unwrap());
		table.insert(67,  to_color("#5f87af").unwrap());
		table.insert(68,  to_color("#5f87d7").unwrap());
		table.insert(69,  to_color("#5f87ff").unwrap());
		table.insert(70,  to_color("#5faf00").unwrap());
		table.insert(71,  to_color("#5faf5f").unwrap());
		table.insert(72,  to_color("#5faf87").unwrap());
		table.insert(73,  to_color("#5fafaf").unwrap());
		table.insert(74,  to_color("#5fafd7").unwrap());
		table.insert(75,  to_color("#5fafff").unwrap());
		table.insert(76,  to_color("#5fd700").unwrap());
		table.insert(77,  to_color("#5fd75f").unwrap());
		table.insert(78,  to_color("#5fd787").unwrap());
		table.insert(79,  to_color("#5fd7af").unwrap());
		table.insert(80,  to_color("#5fd7d7").unwrap());
		table.insert(81,  to_color("#5fd7ff").unwrap());
		table.insert(82,  to_color("#5fff00").unwrap());
		table.insert(83,  to_color("#5fff5f").unwrap());
		table.insert(84,  to_color("#5fff87").unwrap());
		table.insert(85,  to_color("#5fffaf").unwrap());
		table.insert(86,  to_color("#5fffd7").unwrap());
		table.insert(87,  to_color("#5fffff").unwrap());
		table.insert(88,  to_color("#870000").unwrap());
		table.insert(89,  to_color("#87005f").unwrap());
		table.insert(90,  to_color("#870087").unwrap());
		table.insert(91,  to_color("#8700af").unwrap());
		table.insert(92,  to_color("#8700d7").unwrap());
		table.insert(93,  to_color("#8700ff").unwrap());
		table.insert(94,  to_color("#875f00").unwrap());
		table.insert(95,  to_color("#875f5f").unwrap());
		table.insert(96,  to_color("#875f87").unwrap());
		table.insert(97,  to_color("#875faf").unwrap());
		table.insert(98,  to_color("#875fd7").unwrap());
		table.insert(99,  to_color("#875fff").unwrap());
		table.insert(100, to_color("#878700").unwrap());
		table.insert(101, to_color("#87875f").unwrap());
		table.insert(102, to_color("#878787").unwrap());
		table.insert(103, to_color("#8787af").unwrap());
		table.insert(104, to_color("#8787d7").unwrap());
		table.insert(105, to_color("#8787ff").unwrap());
		table.insert(106, to_color("#87af00").unwrap());
		table.insert(107, to_color("#87af5f").unwrap());
		table.insert(108, to_color("#87af87").unwrap());
		table.insert(109, to_color("#87afaf").unwrap());
		table.insert(110, to_color("#87afd7").unwrap());
		table.insert(111, to_color("#87afff").unwrap());
		table.insert(112, to_color("#87d700").unwrap());
		table.insert(113, to_color("#87d75f").unwrap());
		table.insert(114, to_color("#87d787").unwrap());
		table.insert(115, to_color("#87d7af").unwrap());
		table.insert(116, to_color("#87d7d7").unwrap());
		table.insert(117, to_color("#87d7ff").unwrap());
		table.insert(118, to_color("#87ff00").unwrap());
		table.insert(119, to_color("#87ff5f").unwrap());
		table.insert(120, to_color("#87ff87").unwrap());
		table.insert(121, to_color("#87ffaf").unwrap());
		table.insert(122, to_color("#87ffd7").unwrap());
		table.insert(123, to_color("#87ffff").unwrap());
		table.insert(124, to_color("#af0000").unwrap());
		table.insert(125, to_color("#af005f").unwrap());
		table.insert(126, to_color("#af0087").unwrap());
		table.insert(127, to_color("#af00af").unwrap());
		table.insert(128, to_color("#af00d7").unwrap());
		table.insert(129, to_color("#af00ff").unwrap());
		table.insert(130, to_color("#af5f00").unwrap());
		table.insert(131, to_color("#af5f5f").unwrap());
		table.insert(132, to_color("#af5f87").unwrap());
		table.insert(133, to_color("#af5faf").unwrap());
		table.insert(134, to_color("#af5fd7").unwrap());
		table.insert(135, to_color("#af5fff").unwrap());
		table.insert(136, to_color("#af8700").unwrap());
		table.insert(137, to_color("#af875f").unwrap());
		table.insert(138, to_color("#af8787").unwrap());
		table.insert(139, to_color("#af87af").unwrap());
		table.insert(140, to_color("#af87d7").unwrap());
		table.insert(141, to_color("#af87ff").unwrap());
		table.insert(142, to_color("#afaf00").unwrap());
		table.insert(143, to_color("#afaf5f").unwrap());
		table.insert(144, to_color("#afaf87").unwrap());
		table.insert(145, to_color("#afafaf").unwrap());
		table.insert(146, to_color("#afafd7").unwrap());
		table.insert(147, to_color("#afafff").unwrap());
		table.insert(148, to_color("#afd700").unwrap());
		table.insert(149, to_color("#afd75f").unwrap());
		table.insert(150, to_color("#afd787").unwrap());
		table.insert(151, to_color("#afd7af").unwrap());
		table.insert(152, to_color("#afd7d7").unwrap());
		table.insert(153, to_color("#afd7ff").unwrap());
		table.insert(154, to_color("#afff00").unwrap());
		table.insert(155, to_color("#afff5f").unwrap());
		table.insert(156, to_color("#afff87").unwrap());
		table.insert(157, to_color("#afffaf").unwrap());
		table.insert(158, to_color("#afffd7").unwrap());
		table.insert(159, to_color("#afffff").unwrap());
		table.insert(160, to_color("#d70000").unwrap());
		table.insert(161, to_color("#d7005f").unwrap());
		table.insert(162, to_color("#d70087").unwrap());
		table.insert(163, to_color("#d700af").unwrap());
		table.insert(164, to_color("#d700d7").unwrap());
		table.insert(165, to_color("#d700ff").unwrap());
		table.insert(166, to_color("#d75f00").unwrap());
		table.insert(167, to_color("#d75f5f").unwrap());
		table.insert(168, to_color("#d75f87").unwrap());
		table.insert(169, to_color("#d75faf").unwrap());
		table.insert(170, to_color("#d75fd7").unwrap());
		table.insert(171, to_color("#d75fff").unwrap());
		table.insert(172, to_color("#d78700").unwrap());
		table.insert(173, to_color("#d7875f").unwrap());
		table.insert(174, to_color("#d78787").unwrap());
		table.insert(175, to_color("#d787af").unwrap());
		table.insert(176, to_color("#d787d7").unwrap());
		table.insert(177, to_color("#d787ff").unwrap());
		table.insert(178, to_color("#d7af00").unwrap());
		table.insert(179, to_color("#d7af5f").unwrap());
		table.insert(180, to_color("#d7af87").unwrap());
		table.insert(181, to_color("#d7afaf").unwrap());
		table.insert(182, to_color("#d7afd7").unwrap());
		table.insert(183, to_color("#d7afff").unwrap());
		table.insert(184, to_color("#d7d700").unwrap());
		table.insert(185, to_color("#d7d75f").unwrap());
		table.insert(186, to_color("#d7d787").unwrap());
		table.insert(187, to_color("#d7d7af").unwrap());
		table.insert(188, to_color("#d7d7d7").unwrap());
		table.insert(189, to_color("#d7d7ff").unwrap());
		table.insert(190, to_color("#d7ff00").unwrap());
		table.insert(191, to_color("#d7ff5f").unwrap());
		table.insert(192, to_color("#d7ff87").unwrap());
		table.insert(193, to_color("#d7ffaf").unwrap());
		table.insert(194, to_color("#d7ffd7").unwrap());
		table.insert(195, to_color("#d7ffff").unwrap());
		table.insert(196, to_color("#ff0000").unwrap());
		table.insert(197, to_color("#ff005f").unwrap());
		table.insert(198, to_color("#ff0087").unwrap());
		table.insert(199, to_color("#ff00af").unwrap());
		table.insert(200, to_color("#ff00d7").unwrap());
		table.insert(201, to_color("#ff00ff").unwrap());
		table.insert(202, to_color("#ff5f00").unwrap());
		table.insert(203, to_color("#ff5f5f").unwrap());
		table.insert(204, to_color("#ff5f87").unwrap());
		table.insert(205, to_color("#ff5faf").unwrap());
		table.insert(206, to_color("#ff5fd7").unwrap());
		table.insert(207, to_color("#ff5fff").unwrap());
		table.insert(208, to_color("#ff8700").unwrap());
		table.insert(209, to_color("#ff875f").unwrap());
		table.insert(210, to_color("#ff8787").unwrap());
		table.insert(211, to_color("#ff87af").unwrap());
		table.insert(212, to_color("#ff87d7").unwrap());
		table.insert(213, to_color("#ff87ff").unwrap());
		table.insert(214, to_color("#ffaf00").unwrap());
		table.insert(215, to_color("#ffaf5f").unwrap());
		table.insert(216, to_color("#ffaf87").unwrap());
		table.insert(217, to_color("#ffafaf").unwrap());
		table.insert(218, to_color("#ffafd7").unwrap());
		table.insert(219, to_color("#ffafff").unwrap());
		table.insert(220, to_color("#ffd700").unwrap());
		table.insert(221, to_color("#ffd75f").unwrap());
		table.insert(222, to_color("#ffd787").unwrap());
		table.insert(223, to_color("#ffd7af").unwrap());
		table.insert(224, to_color("#ffd7d7").unwrap());
		table.insert(225, to_color("#ffd7ff").unwrap());
		table.insert(226, to_color("#ffff00").unwrap());
		table.insert(227, to_color("#ffff5f").unwrap());
		table.insert(228, to_color("#ffff87").unwrap());
		table.insert(229, to_color("#ffffaf").unwrap());
		table.insert(230, to_color("#ffffd7").unwrap());
		table.insert(231, to_color("#ffffff").unwrap());

		// Grayscale.
		table.insert(232, to_color("#080808").unwrap());
		table.insert(233, to_color("#121212").unwrap());
		table.insert(234, to_color("#1c1c1c").unwrap());
		table.insert(235, to_color("#262626").unwrap());
		table.insert(236, to_color("#303030").unwrap());
		table.insert(237, to_color("#3a3a3a").unwrap());
		table.insert(238, to_color("#444444").unwrap());
		table.insert(239, to_color("#4e4e4e").unwrap());
		table.insert(240, to_color("#585858").unwrap());
		table.insert(241, to_color("#606060").unwrap());
		table.insert(242, to_color("#666666").unwrap());
		table.insert(243, to_color("#767676").unwrap());
		table.insert(244, to_color("#808080").unwrap());
		table.insert(245, to_color("#8a8a8a").unwrap());
		table.insert(246, to_color("#949494").unwrap());
		table.insert(247, to_color("#9e9e9e").unwrap());
		table.insert(248, to_color("#a8a8a8").unwrap());
		table.insert(249, to_color("#b2b2b2").unwrap());
		table.insert(250, to_color("#bcbcbc").unwrap());
		table.insert(251, to_color("#c6c6c6").unwrap());
		table.insert(252, to_color("#d0d0d0").unwrap());
		table.insert(253, to_color("#dadada").unwrap());
		table.insert(254, to_color("#e4e4e4").unwrap());
		table.insert(255, to_color("#eeeeee").unwrap());

		Color {
			table: table,
		}
	}
}

impl Color {
	pub fn load(&mut self, table: &toml::Table) {
		for (key, value) in table {
			if let Ok(index) = key.parse::<u8>() {
				if let Some(color) = value.as_str().and_then(|v| to_color(v)) {
					self.table.insert(index, color);
				}
			}
		}
	}

	pub fn get(&self, index: u8) -> &Rgba<f64> {
		&self.table[&index]
	}
}
