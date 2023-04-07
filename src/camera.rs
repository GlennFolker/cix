use bevy::{
	prelude::*,
	core_pipeline::{
		core_2d::graph::NAME,
		tonemapping::{
		    Tonemapping, DebandDither,
	    },
	},
	math::DVec2,
	render::{
		camera::{
		    CameraRenderGraph,
		    CameraProjection, Viewport,
	    },
	    primitives::Frustum,
	    view::VisibleEntities,
	},
	window::PrimaryWindow,
};

pub const CAMERA_VIEWPORT: DVec2 = DVec2::new(1280., 800.);

#[derive(Resource, Deref, DerefMut, Copy, Clone)]
pub struct CameraPos(pub Vec2);

#[derive(Bundle)]
pub struct CameraFixed2dBundle {
    pub camera: Camera,
    pub camera_render_graph: CameraRenderGraph,
    pub projection: FixedOrthographicProjection,
    pub visible_entities: VisibleEntities,
    pub frustum: Frustum,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub camera_2d: Camera2d,
    pub tonemapping: Tonemapping,
    pub deband_dither: DebandDither,
}

impl Default for CameraFixed2dBundle {
	fn default() -> Self {
		let far = 1000.;
        let projection = OrthographicProjection {
            far,
            ..Default::default()
        };

        let transform = Transform::from_xyz(0.0, 0.0, far - 0.1);
        let view_projection = projection.get_projection_matrix() * transform.compute_matrix().inverse();
        let frustum = Frustum::from_view_projection_custom_far(
            &view_projection,
            &transform.translation,
            &transform.back(),
            projection.far(),
        );

        Self {
            camera_render_graph: CameraRenderGraph::new(NAME),
            projection: FixedOrthographicProjection {
            	ortho: projection,
            	size: CAMERA_VIEWPORT,
            	offset: default(),
            },
            visible_entities: default(),
            frustum,
            transform,
            global_transform: default(),
            camera: default(),
            camera_2d: default(),
            tonemapping: Tonemapping::None,
            deband_dither: DebandDither::Disabled,
        }
	}
}

#[derive(Component, Clone, Default, Reflect, FromReflect)]
#[reflect(Component, Default)]
pub struct FixedOrthographicProjection {
    pub ortho: OrthographicProjection,
    pub size: DVec2,
    pub offset: Vec2,
}

impl CameraProjection for FixedOrthographicProjection {
	#[inline]
    fn update(&mut self, _: f32, _: f32) {
        self.ortho.update(self.size.x as f32, self.size.y as f32);
    }

	#[inline]
	fn get_projection_matrix(&self) -> Mat4 {
		self.ortho.get_projection_matrix()
	}

    #[inline]
    fn far(&self) -> f32 {
    	self.ortho.far()
    }
}

pub fn camera_viewport_sys(
    camera_pos: Res<CameraPos>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
    windows: Query<(Entity, &Window)>,
    images: Res<Assets<Image>>,
    mut camera: Query<(&mut Camera, &mut FixedOrthographicProjection, &mut Transform)>,
) {
	let Ok((mut camera, mut proj, mut trns)) = camera.get_single_mut() else { return };
	trns.translation.x = camera_pos.x;
	trns.translation.y = camera_pos.y;

    if
        let Some(target) = camera.target.normalize(primary_window.iter().next()) &&
        let Some(info) = target.get_render_target_info(&windows, &images)
    {
    	let scl = info.scale_factor;
    	let logical_width = info.physical_size.x as f64 / scl;
    	let logical_height = info.physical_size.y as f64 / scl;

    	let viewport_scale = (logical_width / proj.size.x).min(logical_height / proj.size.y);
    	let viewport = proj.size * viewport_scale;

    	let left = ((logical_width - viewport.x) * scl / 2.).round();
        let top = ((logical_height - viewport.y) * scl / 2.).round();
        let width = (viewport.x * scl).round() as u32;
        let height = (viewport.y * scl).round() as u32;

        let offset = -Vec2::new(left as f32, top as f32);
        let viewport = Viewport {
        	physical_position: UVec2::new(left as u32, top as u32),
        	physical_size: UVec2::new(width, height),
        	depth: default(),
        };

        if proj.offset != offset {
        	proj.offset = offset;
        }

        if let Some(ref prev) = camera.viewport && (
            prev.physical_position != viewport.physical_position ||
            prev.physical_size != viewport.physical_size ||
            prev.depth != viewport.depth
        ) {
        	camera.viewport = Some(viewport);
        } else if camera.viewport.is_none() {
        	camera.viewport = Some(viewport);
        }
    }
}
