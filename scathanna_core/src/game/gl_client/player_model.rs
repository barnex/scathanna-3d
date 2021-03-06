use super::internal::*;

pub struct PlayerModels {
	models: [PlayerModel; 7],
}

const HEAD_PITCH_FACTOR: f32 = 0.25;

impl PlayerModels {
	// TODO: lazy load?
	pub fn new(engine: &Engine) -> Result<Self> {
		Ok(Self {
			models: [
				PlayerModel::frog(engine)?, //
				PlayerModel::panda(engine)?,
				PlayerModel::turkey(engine)?,
				PlayerModel::pig(engine)?,
				PlayerModel::hamster(engine)?,
				PlayerModel::chicken(engine)?,
				PlayerModel::bunny(engine)?,
			],
		})
	}

	pub fn get(&self, avatar_id: u8) -> &PlayerModel {
		// avatar_id gets checked higher up so should be valid.
		// But just in case, return a default if invalid nevertheless.
		self.models.get(avatar_id as usize).unwrap_or(&self.models[0])
	}
}

pub fn parse_avatar_id(s: &str) -> Result<u8> {
	let opts = ["frog", "panda", "turkey", "pig", "hamster", "chicken", "bunny"];
	match s.parse() {
		Ok(v) => Ok(v),
		Err(_) => opts //
			.iter()
			.position(|&v| v == s)
			.map(|v| v as u8)
			.ok_or(anyhow!("avatar options: {}", opts.join(","))),
	}
}

/// Models (on the GPU) needed to draw player avatars.
pub struct PlayerModel {
	head: Rc<VertexArray>,
	foot: Rc<VertexArray>,
	texture: Rc<Texture>,
	gun: (Rc<VertexArray>, Rc<Texture>),
	head_height: f32,
	head_scale: f32,
	foot_scale: f32,
	foot_sep: f32,
}

fn gun(engine: &Engine) -> Result<(Rc<VertexArray>, Rc<Texture>)> {
	Ok((engine.wavefront_obj("bubblegun")?, engine.texture("party_hat", YELLOW)))
}

impl PlayerModel {
	pub fn frog(engine: &Engine) -> Result<Self> {
		Ok(Self {
			head: engine.wavefront_obj("froghead")?,
			foot: engine.wavefront_obj("frogfoot")?,
			texture: engine.texture("frog", GREEN),
			gun: gun(engine)?,
			head_height: 2.0,
			head_scale: 4.0,
			foot_scale: 2.5,
			foot_sep: 0.15,
		})
	}

	pub fn panda(engine: &Engine) -> Result<Self> {
		Ok(Self {
			head: engine.wavefront_obj("pandahead")?,
			foot: engine.wavefront_obj("frogfoot")?,
			texture: engine.texture("panda", WHITE),
			gun: gun(engine)?,
			head_height: 1.5,
			head_scale: 4.2,
			foot_scale: 1.6,
			foot_sep: 0.05,
		})
	}

	pub fn pig(engine: &Engine) -> Result<Self> {
		Ok(Self {
			head: engine.wavefront_obj("pighead")?,
			foot: engine.wavefront_obj("simple_foot")?,
			texture: engine.texture("pig", WHITE),
			gun: gun(engine)?,
			head_height: 1.5,
			head_scale: 4.2,
			foot_scale: 1.6,
			foot_sep: 0.05,
		})
	}

	pub fn turkey(engine: &Engine) -> Result<Self> {
		Ok(Self {
			head: engine.wavefront_obj("turkeyhead")?,
			foot: engine.wavefront_obj("chickenleg")?,
			texture: engine.texture("turkey", WHITE),
			gun: gun(engine)?,
			head_height: 1.5,
			head_scale: 4.2,
			foot_scale: 3.0,
			foot_sep: 0.05,
		})
	}

	pub fn hamster(engine: &Engine) -> Result<Self> {
		Ok(Self {
			head: engine.wavefront_obj("hamsterhead")?,
			foot: engine.wavefront_obj("simple_foot")?,
			texture: engine.texture("hamster", WHITE),
			gun: gun(engine)?,
			head_height: 1.5,
			head_scale: 4.2,
			foot_scale: 1.6,
			foot_sep: 0.05,
		})
	}

	pub fn chicken(engine: &Engine) -> Result<Self> {
		Ok(Self {
			head: engine.wavefront_obj("chickenhead")?,
			foot: engine.wavefront_obj("chickenleg")?,
			texture: engine.texture("chicken", WHITE),
			gun: gun(engine)?,
			head_height: 1.5,
			head_scale: 4.2,
			foot_scale: 2.8,
			foot_sep: 0.05,
		})
	}

	pub fn bunny(engine: &Engine) -> Result<Self> {
		Ok(Self {
			head: engine.wavefront_obj("bunnyhead")?,
			foot: engine.wavefront_obj("simple_foot")?,
			texture: engine.texture("bunny", WHITE),
			gun: gun(engine)?,
			head_height: 1.5,
			head_scale: 4.2,
			foot_scale: 1.6,
			foot_sep: 0.05,
		})
	}

	/// Draw player model as seen by others.
	pub fn draw_3rd_person(&self, engine: &Engine, player: &Player) {
		self.draw_head(engine, player);
		self.draw_feet(engine, player);
		self.draw_gun(engine, player, 1.0 /* sun_intens*/);

		if DBG_GEOMETRY {
			engine.draw_boundingbox(player.skeleton.bounds());
		}
	}

	/// Draw player model as seen by self.
	pub fn draw_1st_person(&self, engine: &Engine, player: &Player, sun_intens: f32) {
		self.draw_feet(engine, player);
		self.draw_gun(engine, player, sun_intens);
	}

	fn draw_gun(&self, engine: &Engine, player: &Player, sun_intens: f32) {
		let scale_mat = scale_matrix(4.5);
		let Orientation { yaw, pitch } = player.orientation();
		let pitch_mat = pitch_matrix(-pitch);
		let hand_mat = translation_matrix(player.gun_pos_internal());
		let yaw_mat = yaw_matrix(-yaw);
		let pos_mat = translation_matrix(player.position());

		let transf = &pos_mat * &yaw_mat * &hand_mat * &pitch_mat * &scale_mat;
		let ambient = 0.3;

		engine.use_texture(&self.gun.1);
		engine.shaders().use_glossy(engine.sun_direction(), sun_intens, ambient, &transf);
		engine.draw_triangles(&self.gun.0);
	}

	fn draw_head(&self, engine: &Engine, player: &Player) {
		let Orientation { yaw, pitch } = player.orientation();
		let head_pos = self.head_height * vec3::EY;
		let transf = translation_matrix(player.position() + head_pos) * yaw_matrix(-yaw) * pitch_matrix(-pitch * HEAD_PITCH_FACTOR) * scale_matrix(self.head_scale);

		let ambient = 0.3;
		let sun_intens = 1.0;

		engine.use_texture(&self.texture);
		engine.shaders().use_glossy(engine.sun_direction(), sun_intens, ambient, &transf);
		engine.draw_triangles(&self.head);
	}

	pub fn draw_hat(&self, engine: &Engine, player: &Player, hat: &Model) {
		let Orientation { yaw, pitch } = player.orientation();
		let pitch_mat = pitch_matrix(-pitch * HEAD_PITCH_FACTOR);
		let top_mat = translation_matrix((self.head_height + 0.75 * self.head_scale) * vec3::EY);

		let yaw_mat = yaw_matrix(-yaw);
		let pos_mat = translation_matrix(player.position());
		let transf = &pos_mat * &yaw_mat * &pitch_mat * &top_mat;
		engine.draw_model_with(hat, &transf);
	}

	fn draw_feet(&self, engine: &Engine, player: &Player) {
		let scale_mat = scale_matrix(self.foot_scale);
		let pitch_mat = pitch_matrix(player.local.feet_pitch);
		let [left_mat, right_mat] = self.feet_pos_internal(player).map(translation_matrix);
		let yaw_mat = yaw_matrix(-player.orientation().yaw);
		let pos_mat = translation_matrix(player.position());

		let transf_l = &pos_mat * &yaw_mat * &left_mat * &pitch_mat * &scale_mat;
		let transf_r = &pos_mat * &yaw_mat * &right_mat * &pitch_mat * &scale_mat;

		let ambient = 0.3;
		let sun_intens = 1.0;

		engine.use_texture(&self.texture);

		engine.shaders().use_glossy(engine.sun_direction(), sun_intens, ambient, &transf_l);
		engine.draw_triangles(&self.foot);

		engine.shaders().use_glossy(engine.sun_direction(), sun_intens, ambient, &transf_r);
		engine.draw_triangles(&self.foot);
	}

	fn feet_pos_internal(&self, player: &Player) -> [vec3; 2] {
		let anim_r = 1.0;
		let c = anim_r * player.local.feet_phase.cos();
		let s = anim_r * player.local.feet_phase.sin();
		[
			vec3(-0.35 * player.skeleton.hsize, f32::max(0.0, s), c) - self.foot_sep * vec3::EX,
			vec3(0.35 * player.skeleton.hsize, f32::max(0.0, -s), -c) + self.foot_sep * vec3::EX,
		]
	}
}
