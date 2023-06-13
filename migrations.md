 Migration Guide: 0.4 to 0.5
"commands: &mut Commands" SystemParam is now "mut commands: Commands" #

// 0.4
fn foo(commands: &mut Commands) {
}

// 0.5
fn foo(mut commands: Commands) {
}

Systems using the old commands: &mut Commands syntax in 0.5 will fail to compile when calling foo.system().

This change was made because Commands now holds an internal World reference to enable safe entity allocations.

Note: The internal World reference requires two lifetime parameters to pass Commands into a non-system function: commands: &'a mut Commands<'b>
Commands API #

The Commands API has been completely reworked for consistency with the World API.

// 0.4
commands
    .spawn(SomeBundle)
    .with(SomeComponent)
    .spawn(SomeBundle); // this sort of chaining is no longer possible

let entity = commands.spawn(SomeBundle).current_entity().unwrap();

commands.despawn(entity);

// 0.5
commands
    .spawn()
    .insert_bundle(SomeBundle)
    .insert(Component);

let entity = commands.spawn().insert_bundle(SomeBundle).id();

commands.entity(entity).despawn();

commands.spawn() no longer accepts any parameters. To spawn bundles, use commands.spawn_bundle(bundle).

Similarly, rather than using with(some_component) to spawn an object with multiple components, you must now use insert(some_component):

// 0.4
commands.spawn(some_bundle)
    .with(some_component);
    
// 0.5
commands.spawn_bundle(some_bundle)
    .insert(some_component);
    
// or...
commands.spawn()
    .insert_bundle(some_bundle)
    .insert(some_component);

Removing and adding components on entities has also been changed:

// 0.4
commands.insert_one(some_entity, SomeComponent);
commands.remove_one::<SomeComponent>(some_entity);

// 0.5
commands.entity(some_entity).insert(SomeComponent);
commands.entity(some_entity).remove::<SomeComponent>();

Timer now uses Duration #

// 0.4
if timer.tick(time.delta_seconds()).finished() { /* do stuff */ }
timer.elapsed() // returns an `f32`

// 0.5
if timer.tick(time.delta()).finished() { /* do stuff */ }
timer.elapsed() // returns a `Duration`

Most of the methods of Timer now use Duration instead of f32.

This change allows timers to have consistent, high precision. For convenience, there is also an elapsed_secs method that returns f32. Otherwise, when you need an f32, use the as_secs_f32() method on Duration to make the conversion.
Simplified Events #

// 0.4
fn event_reader_system(
    mut my_event_reader: Local<EventReader<MyEvent>>,
    my_events: Res<Events<MyEvent>>,
) {
    for my_event in my_event_reader.iter(&my_events) {
        // do things with your event
    }
}

// 0.5
fn event_reader_system(mut my_event_reader: EventReader<MyEvent>) {
    for my_event in my_event_reader.iter() {
        // do things with your event
    }
}

You no longer need two system parameters to read your events. One EventReader is sufficient.

Following the above example of using an EventReader to read events, you can now use EventWriter to create new ones.

// 0.4
fn event_writer_system(
    mut my_events: ResMut<Events<MyEvent>>,
) {
    my_events.send(MyEvent);
}

// 0.5
fn event_writer_system(
    mut my_events: EventWriter<MyEvent>
) {
    my_events.send(MyEvent);
}

AppBuilder::add_resource is now called AppBuilder::insert_resource #

This is a small change to have function names on AppBuilder consistent with the Commands API.
TextBundle #

This bundle has been reworked to allow multiple differently-styled sections of text within a single bundle. Text::with_section was added to simplify the common case where you're only interested in one text section.

// 0.4
TextBundle {
    text: Text {
        value: "hello!".to_string(),
        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
        style: TextStyle {
            font_size: 60.0,
            color: Color::WHITE,
            ..Default::default()
        },
    },
    ..Default::default()
}

// 0.5
TextBundle {
    text: Text::with_section(
        "hello!",
        TextStyle {
            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
            font_size: 60.0,
            color: Color::WHITE,
        },
        TextAlignment::default()
    ),
    ..Default::default()
}

Scene must now be specified when loading a GLTF scene #

Previously, you were able to load a GLTF scene asset with only a path. Now, you must include a fragment specifying the scene you want to load. If you only have one scene in the file, it's #Scene0.

// 0.4
asset_server.load("models/foo.glb");

// 0.5
asset_server.load("models/foo.glb#Scene0");

State #

States are now registered with AppBuilder::add_state, which creates the State resource and registers a "driver" system that takes the place of StateStage. States are registered using SystemSet.

IMPORTANT: if you stop registering the StateStage but don't register the driver (using add_state or State::get_driver), Bevy 0.5 will enter an infinite loop, causing your application to "lock up".

// 0.4
app.insert_resource(State::new(MyState::InitState))
   .add_stage_after(
       bevy::app::stage::UPDATE,
       MY_STATE_STAGE_NAME,
       bevy::ecs::StateStage::<MyState>::default(),
   )
   .on_state_enter(
       MY_STATE_STAGE_NAME,
       MyState::InitState,
       enter_init_state.system())
   .on_state_update(
       MY_STATE_STAGE_NAME,
       MyState::InitState,
       update_init_state.system())
   .on_state_exit(
       MY_STATE_STAGE_NAME,
       MyState::InitState,
       exit_init_state.system());

// 0.5
app.add_state(MyState::InitState)
   .add_system_set(SystemSet::on_enter(MyState::InitState)
       .with_system(enter_init_state.system()))
   .add_system_set(SystemSet::on_update(MyState::InitState)
       .with_system(update_init_state.system()))
   .add_system_set(SystemSet::on_exit(MyState::InitState)
       .with_system(exit_init_state.system()));

It is still possible to register the driver manually using State::get_driver, but this is not normally required.
ChangedRes removed #

This change was made to allow for more flexiblity and more consistent behavior with change detection for components.

// 0.4
fn some_system(
    res: ChangedRes<SomeResource>
) {
    // this system only runs if SomeResource has changed
}

// 0.5
fn some_system(
    res: Res<SomeResource> // or ResMut
) {
    // this system always runs

    if !res.is_changed() { // or .is_added()
        return;
    }
}

Cameras #

Camera3dBundle is now known as PerspectiveCameraBundle, and Camera2dBundle is now known as OrthographicCameraBundle.

OrthographicCameraBundle does not implement Default, so to change its transform at spawn while keeping everything else the same, consider something like the following:

let mut camera = OrthographicCameraBundle::new_2d();
camera.transform = Transform::from_translation(Vec3::new(0.0, 0.0, 5.0));
commands.spawn_bundle(camera);

Render API changes #

RasterizationStateDescriptor no longer exists. Much of its functionality has been moved to other fields on PipelineDescriptor. cull_mode, for example, is now found in the primitive: PrimitiveState field.

Buffers of type Vec<Color> can no longer be uploaded to the GPU directly due to limitations with RenderResources and the new Byteable requirement. Consider using a Vec<Vec4> instead, and inserting colors with as_rgba_f32() and .into() instead:

#[derive(RenderResources, Default, TypeUuid)]
struct SomeShader {
    #[render_resources(buffer)]
    pub colors: Vec<Vec4>
}

fn add_some_color(shader: SomeShader, color: Color) {
    shader.colors.push(color.as_rgba_f32().into());
}

Shaders should now use CameraViewProj #

The ViewProj matrix is now set via the name CameraViewProj rather than Camera. If you don't update this, bevy will fail silently and you won't be able to see anything!

// 0.4
layout(set = 0, binding = 0) uniform Camera {
    mat4 ViewProj;
}

// 0.5
layout(set = 0, binding = 0) uniform CameraViewProj {
    mat4 ViewProj;
}

Diagnostics #

PrintDiagnosticsPlugin is now LogDiagnosticsPlugin.
System Ordering #

The parallel system executor has been redesigned. Systems that had implicit orderings might no longer run in the same order. For more detail on the new behavior, see the release notes.


Migration Guide: 0.5 to 0.6
Rust 2021 now required #

Bevy has been updated to use Rust 2021. This means we can take advantage of the new Cargo feature resolver by default (which both Bevy and the new wgpu version require). Make sure you update your crates to Rust 2021 or you will need to manually enable the new feature resolver with `resolver = "2" in your Cargo.toml.

[package]
name = "your_app"
version = "0.1.0"
edition = "2021"

Note that "virtual Cargo workspaces" still need to manually define resolver = "2", even in Rust 2021. Refer to the Rust 2021 documentation for details.

[workspace]
resolver = "2" # Important! wgpu/Bevy needs this!
members = [ "my_crate1", "my_crate2" ]

"AppBuilder" was merged into "App" #

All functions of AppBuilder were merged into App .

In practice this means that you start constructing an App by calling App::new() instead of App::build() and Plugin::build() takes a App instead of a AppBuilder

// 0.5
fn main() {
    App::build()
        .add_plugin(SomePlugin)
        .run();
}

impl Plugin for SomePlugin {
    fn build(&self, app: &mut AppBuilder) {

    }
}

// 0.6
fn main() {
    App::new()
        .add_plugin(SomePlugin)
        .run();
}

impl Plugin for SomePlugin {
    fn build(&self, app: &mut App) {

    }
}

The "Component" trait now needs to be derived #

Bevy no longer has a blanket implementation for the Component trait. Instead you need to derive (or manualy implement) the trait for every Type that needs it.

// 0.5
struct MyComponent;

// 0.6
#[derive(Component)]
struct MyComponent;

In order to use foreign types as components, wrap them using the newtype pattern.

#[derive(Component)]
struct Cooldown(std::time::Duration);

Setting the Component Storage is now done in "Component" Trait #

The change to deriving Component , enabled setting the Component Storage at compiletime instead of runtime.

// 0.5
appbuilder
    .world
    .register_component(ComponentDescriptor::new::<MyComponent>(
        StorageType::SparseSet,
    ))
    .unwrap();

// 0.6
#[derive(Component)]
#[component(storage = "SparseSet")]
struct MyComponent;

Calling ".system()" on a system is now optional #

When adding a system to Bevy it is no longer necessary to call .system() beforehand.

// 0.5
fn main() {
    App::new()
        .add_system(first_system.system())
        .add_system(second_system.system())
        .run();
}

// 0.6
fn main() {
    App::new()
        .add_system(first_system)
        .add_system(second_system.system())
        .run();
}

System configuration Functions like .label() or .config() can now also be directly called on a system.

// 0.5
fn main() {
    App::new()
        .add_system(first_system.system().label("FirstSystem"))
        .add_system(second_system.system().after("FirstSystem"))
        .run();
}

// 0.6
fn main() {
    App::new()
        .add_system(first_system.label("FirstSystem"))
        .add_system(second_system.after("FirstSystem"))
        .run();
}

".single()" and ".single_mut()" are now infallible #

The functions Query::single() and Query::single_mut() no longer return a Result and Panic instead, if not exactly one Entity was found.

If you need the old behavior you can use the fallible Query::get_single() and Query::get_single_mut() instead.

// 0.5
fn player_system(query: Query<&Transform, With<Player>>) {
    let player_position = query.single().unwrap();
    // do something with player_position
}

// 0.6
fn player_system_infallible(query: Query<&Transform, With<Player>>) {
    let player_position = query.single();
    // do something with player_position
}

fn player_system_fallible(query: Query<&Transform, With<Player>>) {
    let player_position = query.get_single().unwrap();
    // do something with player_position
}

"Light" and "LightBundle" are now "PointLight" and "PointLightBundle" #

// 0.5
commands.spawn_bundle(LightBundle {
    light: Light {
        color: Color::rgb(1.0, 1.0, 1.0),
        depth: 0.1..50.0,
        fov: f32::to_radians(60.0),
        intensity: 200.0,
        range: 20.0,
    },
    ..Default::default()
});

// 0.6
commands.spawn_bundle(PointLightBundle {
    light: PointLight {
        color: Color::rgb(1.0, 1.0, 1.0),
        intensity: 200.0,
        range: 20.0,
    },
    ..Default::default()
});

The Light and LightBundle were renamed to PointLight and PointLightBundle to more clearly communicate the behavior of the Light Source. At the same time the fov and depth fields were removed from the PointLight as they were unused.
System Param Lifetime Split #

The Lifetime of SystemParam was split in two separate Lifetimes.

// 0.5
type SystemParamAlias<'a> = (Res<'a, AssetServer>, Query<'a, &'static Transform>, Local<'a, i32>);

#[derive(SystemParam)]
struct SystemParamDerive<'a> {
    res: Res<'a, AssetServer>,
    query: Query<'a, &Transform>,
    local: Local<'a, i32>,
}

// 0.6
type SystemParamAlias<'w, 's> = (Res<'w, AssetServer>, Query<'w, 's, &'static Transform>, Local<'s, i32>);

#[derive(SystemParam)]
struct SystemParamDerive<'w, 's> {
    res: Res<'w, AssetServer>,
    query: Query<'w, 's, &'static Transform>,
    local: Local<'s, i32>,
}

QuerySet declare "QueryState" instead of "Query" #

Due to the System Param Lifetime Split, QuerySets now need to specify their Queries with QueryState instead of Query .

// 0.5
fn query_set(mut queries: QuerySet<(Query<&mut Transform>, Query<&Transform>)>) {

}

// 0.6
fn query_set(mut queries: QuerySet<(QueryState<&mut Transform>, QueryState<&Transform>)>) {

}

"Input<T>.update()" is renamed to "Input<T>.clear()" #

The Input::update() function was renamed to Input::clear() .
"SystemState" is now "SystemMeta" #

The SystemState struct, which stores the metadata of a System, was renamed to SystemMeta .

This was done to accommodate the new SystemState which allows easier cached access to SystemParams outside of a regular System.
Vector casting functions are now named to match return type #

The casting functions for IVec2 , DVec2 , UVec2 , Vec2 have all been changed from being named after their inner elements' cast target to what the entire "Vec" is being casted into. This affects all the different dimensions of the math vectors (i.e., Vec2 , Vec3 and Vec4 ).

// 0.5
let xyz: Vec3 = Vec3::new(0.0, 0.0, 0.0);
let xyz: IVec3 = xyz.as_i32();

// 0.6
let xyz: Vec3 = Vec3::new(0.0, 0.0, 0.0);
let xyz: IVec3 = xyz.as_ivec3();

StandardMaterial's "roughness" is renamed to "perceptual_roughness" #

The StandardMaterial field roughness was renamed to perceptual_roughness.
SpriteBundle and Sprite #

The SpriteBundle now uses a texture handle rather than a material. The color field of the material is now directly available inside of the Sprite struct, which also had its resize_mode field replaced with a custom_size. The following example shows how to spawn a tinted sprite at a particular size. For simpler cases, check out the updated sprite and rect examples.

// 0.5
SpriteBundle {
    sprite: Sprite {
        size: Vec2::new(256.0, 256.0),
        resize_mode: SpriteResizeMode::Manual,
        ..Default::default()
    },
    material: materials.add(ColorMaterial {
        color: Color::RED,
        texture: Some(asset_server.load("branding/icon.png")),
    }),
    ..Default::default()
}

// 0.6
SpriteBundle {
    sprite: Sprite {
        custom_size: Some(Vec2::new(256.0, 256.0)),
        color: Color::RED,
        ..Default::default()
    },
    texture: asset_server.load("branding/icon.png"),
    ..Default::default()
}

Visible is now Visibility #

The Visible struct, which is used in a number of components to set visibility, was renamed to Visibility . Additionally, the field is_transparent was removed from the struct. For 3D, transparency can be set using the alpha_mode field on a material. Transparency is now automatically enabled for all objects in 2D.

// 0.5
let material_handle = materials.add(StandardMaterial {
    base_color_texture: Some(texture.clone()),
    ..Default::default()
});

commands.spawn_bundle(PbrBundle {
    material: material_handle,
    visible: Visible {
        is_visible: true,
        is_transparent: true,
    },
    ..Default::default()
});

// 0.6
let material_handle = materials.add(StandardMaterial {
    base_color_texture: Some(texture.clone()),
    alpha_mode: AlphaMode::Blend,
    ..Default::default()
});

commands.spawn_bundle(PbrBundle {
    material: material_handle,
    visibility: Visibility {
        is_visible: true,
    },
    ..Default::default()
});

Migration Guide: 0.7 to 0.8

Before migrating make sure to run rustup update

Bevy relies heavily on improvements in the Rust language and compiler. As a result, the Minimum Supported Rust Version (MSRV) is "the latest stable release" of Rust.
Camera Driven Rendering #

This is a very complicated change and it is recommended to read the linked PRs for more details

// old 3d perspective camera
commands.spawn_bundle(PerspectiveCameraBundle::default())

// new 3d perspective camera
commands.spawn_bundle(Camera3dBundle::default())

// old 2d orthographic camera
commands.spawn_bundle(OrthographicCameraBundle::new_2d())

// new 2d orthographic camera
commands.spawn_bundle(Camera2dBundle::default())

// old 3d orthographic camera
commands.spawn_bundle(OrthographicCameraBundle::new_3d())

// new 3d orthographic camera
commands.spawn_bundle(Camera3dBundle {
    projection: OrthographicProjection {
        scale: 3.0,
        scaling_mode: ScalingMode::FixedVertical(5.0),
        ..default()
    }.into(),
    ..default()
})

UI no longer requires a dedicated camera. UiCameraBundle has been removed. Camera2dBundle and Camera3dBundle now both default to rendering UI as part of their own render graphs. To disable UI rendering for a camera, disable it using the UiCameraConfig component:

commands
    .spawn_bundle(Camera3dBundle::default())
    .insert(UiCameraConfig {
        show_ui: false,
        ..default()
    })

// 0.7
camera.world_to_screen(transform, world_position);

// 0.8
camera.world_to_viewport(transform, world_position);

Visibilty Inheritance, universal ComputedVisibility and RenderLayers support #

Visibility is now propagated into children in a similar way to Transform. Root elements of a hierarchy must now contain Visibility and ComputedVisiblity for visibility propagation to work.

SpatialBundle and VisibilityBundle have been added for convenience. If you were using a TransformBundle you should probably be using a SpatialBundle now.

If you were previously reading Visibility::is_visible as the "actual visibility" for sprites or lights, use ComputedVisibilty::is_visible() instead:

// 0.7
fn system(query: Query<&Visibility>) {
  for visibility in query.iter() {
    if visibility.is_visible {
       info!("found visible entity");
    }
  }
}

// 0.8
fn system(query: Query<&ComputedVisibility>) {
  for visibility in query.iter() {
    if visibility.is_visible() {
       info!("found visible entity");
    }
  }
}

Use Affine3A for GlobalTransform to allow any affine transformation #

GlobalTransform fields have changed

    Replace global_transform.translation by global_transform.translation() (For other fields, use the compute_transform method)
    GlobalTransform do not support non-linear scales anymore, we'd like to hear from you if it is an inconvenience for you
    If you need the scale, rotation or translation property you can now use global_transform.to_scale_rotation_translation()

// 0.7
let transform = Transform::from(*global_transform);
transform.scale
transform.rotation
transform.translation

// 0.8
let (scale, rotation, translation) = global_transform.to_scale_rotation_translation();

Add a SceneBundle to spawn a scene #

// 0.7
commands.spawn_scene(asset_server.load("models/FlightHelmet/FlightHelmet.gltf#Scene0"));

// 0.8
commands.spawn_bundle(SceneBundle {
    scene: asset_server.load("models/FlightHelmet/FlightHelmet.gltf#Scene0"),
    ..Default::default()
});

The scene will be spawned as a child of the entity with the SceneBundle
Make ScalingMode more flexible #

Adds ability to specify scaling factor for WindowSize, size of the fixed axis for FixedVertical and FixedHorizontal and a new ScalingMode that is a mix of FixedVertical and FixedHorizontal
Allow closing windows at runtime #

bevy::input::system::exit_on_esc_system has been removed. Use bevy::window::close_on_esc instead.

CloseWindow has been removed. Use Window::close instead. The Close variant has been added to WindowCommand. Handle this by closing the relevant window.
Make RunOnce a non-manual System impl #

The run criterion RunOnce, which would make the controlled systems run only once, has been replaced with a new run criterion function ShouldRun::once. Replace all instances of RunOnce with ShouldRun::once.
Move system_param fetch struct into anonymous scope to avoid name collisions #

For code that was using a system param's fetch struct, such as EventReader's EventReaderState, the fetch struct can now be identified via the SystemParam trait associated type Fetch, e.g. for EventReader<T> it can be identified as <EventReader<'static, 'static, T> as SystemParam>::Fetch
Task doesn't impl Component #

If you need a Task to be a Component you should use a wrapper type.

// 0.7
fn system(mut commands: Commands, thread_pool: Res<AsyncComputeTaskPool>) {
    let task = thread_pool.spawn(async move {
        // Complicated async work
        Vec2::ZERO
    });
    commands.spawn().insert(task);
}

// 0.8
#[derive(Component)]
struct ComputeVec2(Task<Vec2>);

fn system(mut commands: Commands) {
    let thread_pool = AsyncComputeTaskPool::get();
    let task = thread_pool.spawn(async move {
        // Complicated async work
        Vec2::ZERO
    });
    commands.spawn().insert(ComputeVec2(task));
}

Split time functionality into bevy_time #

    Time related types (e.g. Time, Timer, Stopwatch, FixedTimestep, etc.) should be imported from bevy::time::* rather than bevy::core::*.
    If you were adding CorePlugin manually, you'll also want to add TimePlugin from bevy::time.
    The bevy::core::CorePlugin::Time system label is replaced with bevy::time::TimeSystem.

Move float_ord from bevy_core to bevy_utils #

Replace imports of bevy::core::FloatOrd with bevy::utils::FloatOrd.
Move Rect to bevy_ui and rename it to UiRect #

The Rect type has been renamed to UiRect.
Rename ElementState to ButtonState #

The ElementState type has been renamed to ButtonState.
Improve docs and naming for RawWindowHandle functionality #

Renamed HasRawWindowHandleWrapper to ThreadLockedRawWindowHandleWrapper.
Migrate to encase from crevice #
Use ShaderType instead of AsStd140 and AsStd430 #

// old
#[derive(AsStd140)]
struct Foo {
    a: Vec4,
    b: Mat4,
}

// new
#[derive(ShaderType)]
struct Foo {
    a: Vec4,
    b: Mat4,
}

StorageBuffer #

    removed set_body(), values(), values_mut(), clear(), push(), append()
    added set(), get(), get_mut()

UniformVec -> UniformBuffer #

    renamed uniform_buffer() to buffer()
    removed len(), is_empty(), capacity(), push(), reserve(), clear(), values()
    added set(), get()

DynamicUniformVec -> DynamicUniformBuffer #

    renamed uniform_buffer() to buffer()
    removed capacity(), reserve()

Make paused timers update just_finished on tick #

Timer::times_finished has been renamed to Timer::times_finished_this_tick for clarity.
Change default Image FilterMode to Linear #

Default Image filtering changed from Nearest to Linear.

// 0.7

// Nothing, nearest was the default

// 0.8
App::new()
    .insert_resource(ImageSettings::default_nearest())

Image.sampler_descriptor has been changed to use ImageSampler instead of SamplerDescriptor.

// 0.7
texture.sampler_descriptor = SamplerDescriptor {
    address_mode_u: AddressMode::Repeat,
    address_mode_v: AddressMode::Repeat,
    ..default()
};

// 0.8
texture.sampler_descriptor = ImageSampler::Descriptor(SamplerDescriptor {
    address_mode_u: AddressMode::Repeat,
    address_mode_v: AddressMode::Repeat,
    ..default()
});

Remove .system() #

You can no longer use .system(). It was deprecated in 0.7.0. You can just remove the method call.

If you needed this for tests purposes, you can use bevy_ecs::system::assert_is_system instead.
Change gamepad.rs tuples to normal structs #

The Gamepad, GamepadButton, GamepadAxis, GamepadEvent and GamepadEventRaw types are now normal structs instead of tuple structs and have a new function. To migrate change every instantiation to use the new() function instead and use the appropriate field names instead of .0 and .1.
Remove EntityMut::get_unchecked #

Replace calls to EntityMut::get_unchecked with calls to EntityMut::get.
Replace ReadOnlyFetch with ReadOnlyWorldQuery #

The trait ReadOnlyFetch has been replaced with ReadOnlyWorldQuery along with the WorldQueryGats::ReadOnlyFetch assoc type which has been replaced with <WorldQuery::ReadOnly as WorldQueryGats>::Fetch

The trait ReadOnlyFetch has been replaced with ReadOnlyWorldQuery along with the WorldQueryGats::ReadOnlyFetch assoc type which has been replaced with <WorldQuery::ReadOnly as WorldQueryGats>::Fetch

    Any where clauses such as QueryFetch<Q>: ReadOnlyFetch should be replaced with Q: ReadOnlyWorldQuery.
    Any custom world query impls should implement ReadOnlyWorldQuery instead of ReadOnlyFetch

Functions update_component_access and update_archetype_component_access have been moved from the FetchState trait to WorldQuery

    Any callers should now call Q::update_component_access(state instead of state.update_component_access (and update_archetype_component_access respectively)
    Any custom world query impls should move the functions from the FetchState impl to WorldQuery impl

WorldQuery has been made an unsafe trait, FetchState has been made a safe trait. (I think this is how it should have always been, but regardless this is definitely necessary now that the two functions have been moved to WorldQuery)

    If you have a custom FetchState impl make it a normal impl instead of unsafe impl
    If you have a custom WorldQuery impl make it an unsafe impl, if your code was sound before it is going to still be sound

Fix unsoundness with Or/AnyOf/Option component access' #

Query conflicts from Or/AnyOf/Option have been fixed, and made stricter to avoid undefined behaviour. If you have new query conflicts due to this you must refactor your systems; consider using ParamSet.
Remove task_pool parameter from par_for_each(_mut) #

The task_pool parameter for Query(State)::par_for_each(_mut) has been removed. Remove these parameters from all calls to these functions.

// 0.7
fn parallel_system(
   task_pool: Res<ComputeTaskPool>,
   query: Query<&MyComponent>,
) {
   query.par_for_each(&task_pool, 32, |comp| {
        // ...
   });
}

// 0.8
fn parallel_system(query: Query<&MyComponent>) {
   query.par_for_each(32, |comp| {
        // ...
   });
}

If using Query or QueryState outside of a system run by the scheduler, you may need to manually configure and initialize a ComputeTaskPool as a resource in the World.
Fail to compile on 16-bit platforms #

bevy_ecs will now explicitly fail to compile on 16-bit platforms, because it is unsound on those platforms due to various internal assumptions.

There is currently no alternative, but we're open to adding support. Please file an issue to help detail your use case.
Enforce type safe usage of Assets::get #

Assets::<T>::get and Assets::<T>::get_mut now require that the passed handles are Handle<T>, improving the type safety of handles. If you were previously passing in:

    a HandleId, use &Handle::weak(id) instead, to create a weak handle. You may have been able to store a type safe Handle instead.
    a HandleUntyped, use &handle_untyped.typed_weak() to create a weak handle of the specified type. This is most likely to be the useful when using load_folder
    a &str or anything not previously mentioned: assets.get(&assets.get_handle("asset/path.ron"))
    a Handle<U> of of a different type, consider whether this is the correct handle type to store. If it is (i.e. the same handle id is used for multiple different Asset types) use Handle::weak(handle.id) to cast to a different type.

Allow higher order systems #

SystemParamFunction has changed. It was not previously part of the public API, so no migration instructions are provided. (It is now included in the public API, although you still should not implement this trait for your own types).

If possible, any custom System implementations should be migrated to use higher order systems, which are significantly less error-prone.

Research is needed into allowing this to work for more cases.
Added offset parameter to TextureAtlas::from_grid_with_padding #

Calls to TextureAtlas::from_grid_with_padding should be modified to include a new parameter, which can be set to Vec2::ZERO to retain old behaviour.

// 0.7
from_grid_with_padding(texture, tile_size, columns, rows, padding)

// 0.8
from_grid_with_padding(texture, tile_size, columns, rows, padding, Vec2::ZERO)

Split mesh shader files #

In shaders for 3D meshes:

    #import bevy_pbr::mesh_view_bind_group -> #import bevy_pbr::mesh_view_bindings
    #import bevy_pbr::mesh_struct -> #import bevy_pbr::mesh_types
        NOTE: If you are using the mesh bind group at bind group index 2, you can remove those binding statements in your shader and just use #import bevy_pbr::mesh_bindings which itself imports the mesh types needed for the bindings.

In shaders for 2D meshes:

    #import bevy_sprite::mesh2d_view_bind_group -> #import bevy_sprite::mesh2d_view_bindings
    #import bevy_sprite::mesh2d_struct -> #import bevy_sprite::mesh2d_types
        NOTE: If you are using the mesh2d bind group at bind group index 2, you can remove those binding statements in your shader and just use #import bevy_sprite::mesh2d_bindings which itself imports the mesh2d types needed for the bindings.

Camera Driven Viewports #

Camera::projection_matrix is no longer a public field. Use the new Camera::projection_matrix() method instead:

// 0.7
let projection = camera.projection_matrix;

// 0.8
let projection = camera.projection_matrix();

Diagnostics: meaningful error when graph node has wrong number of inputs #

Exhaustive matches on RenderGraphRunnerError will need to add a branch to handle the new MismatchedInputCount variant.
Make Reflect safe to implement #

    Reflect derives should not have to change anything
    Manual reflect impls will need to remove the unsafe keyword, add any() implementations, and rename the old any and any_mut to as_any and as_mut_any.
    Calls to any/any_mut must be changed to as_any/as_mut_any

Mark mutable APIs under ECS storage as pub(crate) #

If you experienced any problems caused by this change, please create an issue explaining in detail what you were doing with those apis.
Add global init and get accessors for all newtyped TaskPools #

Thread pools don't need to be stored in a resource anymore since they are now stored globally. You can now use get() to access it.

// 0.7
fn spawn_tasks(thread_pool: Res<AsyncComputeTaskPool>) {
    // Do something with thread_pool
}

// 0.8
fn spawn_tasks() {
    let thread_pool = AsyncComputeTaskPool::get();
    // Do something with thread_pool
}

Simplify design for *Labels #

    Any previous use of Box<dyn SystemLabel> should be replaced with SystemLabelId.
    AsSystemLabel trait has been modified.
        No more output generics.
        Method as_system_label now returns SystemLabelId, removing an unnecessary level of indirection.
    If you need a label that is determined at runtime, you can use Box::leak. Not recommended.

Move get_short_name utility method from bevy_reflect into bevy_utils #

    added bevy_utils::get_short_name, which strips the path from a type name for convenient display.
    removed the TypeRegistry::get_short_name method. Use the function in bevy_utils instead.

Remove dead SystemLabelMarker struct #

This struct had no internal use, docs, or intuitable external use.

It has been removed.
Add reflection for resources #

Rename ReflectComponent::add_component into ReflectComponent::insert_component.
Make reflect_partial_eq return more accurate results #

Previously, all reflect_***_partial_eq helper methods returned Some(false) when the comparison could not be performed, which was misleading. They now return None when the comparison cannot be performed.
Make RenderStage::Extract run on the render world #

The Extract RenderStage now runs on the render world (instead of the main world as before). You must use the Extract SystemParam to access the main world during the extract phase. Extract takes a single type parameter, which is any system parameter (such as Res, Query etc.). It will extract this from the main world. Note that Commands will not work correctly in Extract - it will currently silently do nothing.

// 0.7
fn extract_clouds(mut commands: Commands, clouds: Query<Entity, With<Cloud>>) {
    for cloud in clouds.iter() {
        commands.get_or_spawn(cloud).insert(Cloud);
    }
}

// 0.8
fn extract_clouds(mut commands: Commands, mut clouds: Extract<Query<Entity, With<Cloud>>>) {
    for cloud in clouds.iter() {
        commands.get_or_spawn(cloud).insert(Cloud);
    }
}

You can now also access resources from the render world using the normal system parameters during Extract:

fn extract_assets(mut render_assets: ResMut<MyAssets>, source_assets: Extract<Res<MyAssets>>) {
     *render_assets = source_assets.clone();
}

Because extraction now runs in the render world, usage of Res<RenderWorld> in the main world, should be replaced with usage of Res<MainWorld> in the render world.

Please note that all existing extract systems need to be updated to match this new style; even if they currently compile they will not run as expected. A warning will be emitted on a best-effort basis if this is not met.
Improve Gamepad DPad Button Detection #

D-pad inputs can no longer be accessed as axes. Access them as gamepad buttons instead.
Change window position types from tuple to vec #

Changed the following fields

    WindowCommand::SetWindowMode.resolution from (u32, u32) to UVec2
    WindowCommand::SetResolution.logical_resolution from (f32, f32) to Vec2

Full documentation for bevy_asset #

Rename FileAssetIo::get_root_path to FileAssetIo::get_base_path

FileAssetIo::root_path() is a getter for the root_path field, while FileAssetIo::get_root_path returned the parent directory of the asset root path, which was the executable's directory unless CARGO_MANIFEST_DIR was set. This change solves the ambiguity between the two methods.
Hierarchy commandization #

The Parent and Children component fields are now private.

    Replace parent.0 by parent.get()
    Replace children.0 with *children
    You can't construct Children or Parent component anymore, you can use this as a stopgap measure, which may introduce a single frame delay

#[derive(Component)]
pub struct MakeChildOf(pub Entity);

fn add_parent(
    mut commands: Commands,
    orphans: Query<(Entity, &MakeChildOf)>,
) {
    for (child, MakeChildOf(parent)) in &orphans {
        commands.entity(*parent).add_child(child);
        commands.entity(child).remove::<MakeChildOf>();
    }
}

Remove blanket Serialize + Deserialize requirement for Reflect on generic types #

.register_type for generic types like Option<T>, Vec<T>, HashMap<K, V> will no longer insert ReflectSerialize and ReflectDeserialize type data. Instead you need to register it separately for concrete generic types like so:

    .register_type::<Option<String>>()
    .register_type_data::<Option<String>, ReflectSerialize>()
    .register_type_data::<Option<String>, ReflectDeserialize>()

Lighter no default features #

bevy_asset and bevy_scene are no longer enabled when no-default-features is used with the bevy dependency.

    Crates that use Bevy with no-default-features will need to add these features manually if they rely on them.

bevy = { version = "0.8", default-features = false, features = [
    "bevy_asset",
    "bevy_scene",
] }

Improve ergonomics and reduce boilerplate around creating text elements #

Text::with_section was renamed to Text::from_section and no longer takes a TextAlignment as argument. Use with_alignment to set the alignment instead.
Add QueryState::get_single_unchecked_manual and its family #

Change system::QuerySingleError to query::QuerySingleError
tracing-tracy updated from 0.8.0 to 0.10.0 #

The required tracy version when using the trace-tracy feature is now 0.8.1.

Migration Guide: 0.8 to 0.9

Before migrating make sure to run rustup update

Bevy relies heavily on improvements in the Rust language and compiler. As a result, the Minimum Supported Rust Version (MSRV) is "the latest stable release" of Rust.
Make Resource trait opt-in, requiring #[derive(Resource)] V2 #

Add #[derive(Resource)] to all types you are using as a resource.

If you are using a third party type as a resource, wrap it in a tuple struct to bypass orphan rules. Consider deriving Deref and DerefMut to improve ergonomics.

ClearColor no longer implements Component. Using ClearColor as a component in 0.8 did nothing. Use the ClearColorConfig in the Camera3d and Camera2d components instead.
Plugins own their settings. Rework PluginGroup trait. #

The WindowDescriptor settings have been moved from a resource to WindowPlugin::window:

// Old (Bevy 0.8)
app
  .insert_resource(WindowDescriptor {
    width: 400.0,
    ..default()
  })
  .add_plugins(DefaultPlugins)

// New (Bevy 0.9)
app.add_plugins(DefaultPlugins.set(WindowPlugin {
  window: WindowDescriptor {
    width: 400.0,
    ..default()
  },
  ..default()
}))

The AssetServerSettings resource has been removed in favor of direct AssetPlugin configuration:

// Old (Bevy 0.8)
app
  .insert_resource(AssetServerSettings {
    watch_for_changes: true,
    ..default()
  })
  .add_plugins(DefaultPlugins)

// New (Bevy 0.9)
app.add_plugins(DefaultPlugins.set(AssetPlugin {
  watch_for_changes: true,
  ..default()
}))

add_plugins_with has been replaced by add_plugins in combination with the builder pattern:

// Old (Bevy 0.8)
app.add_plugins_with(DefaultPlugins, |group| group.disable::<AssetPlugin>());

// New (Bevy 0.9)
app.add_plugins(DefaultPlugins.build().disable::<AssetPlugin>());

PluginGroupBuilder and the PluginGroup trait have also been reworked.

// Old (Bevy 0.8)
impl PluginGroup for HelloWorldPlugins {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group.add(PrintHelloPlugin).add(PrintWorldPlugin);
    }
}

// New (Bevy 0.9)
impl PluginGroup for HelloWorldPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(PrintHelloPlugin)
            .add(PrintWorldPlugin)
    }
}

Use plugin setup for resource only used at setup time #

The LogSettings settings have been moved from a resource to LogPlugin configuration:

// Old (Bevy 0.8)
app
  .insert_resource(LogSettings {
    level: Level::DEBUG,
    filter: "wgpu=error,bevy_render=info,bevy_ecs=trace".to_string(),
  })
  .add_plugins(DefaultPlugins)

// New (Bevy 0.9)
app.add_plugins(DefaultPlugins.set(LogPlugin {
    level: Level::DEBUG,
    filter: "wgpu=error,bevy_render=info,bevy_ecs=trace".to_string(),
}))

The ImageSettings settings have been moved from a resource to ImagePlugin configuration:

// Old (Bevy 0.8)
app
  .insert_resource(ImageSettings::default_nearest())
  .add_plugins(DefaultPlugins)

// New (Bevy 0.9)
app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))

The DefaultTaskPoolOptions settings have been moved from a resource to CorePlugin::task_pool_options:

// Old (Bevy 0.8)
app
  .insert_resource(DefaultTaskPoolOptions::with_num_threads(4))
  .add_plugins(DefaultPlugins)

// New (Bevy 0.9)
app.add_plugins(DefaultPlugins.set(CorePlugin {
  task_pool_options: TaskPoolOptions::with_num_threads(4),
}))

Remove AssetServer::watch_for_changes() #

AssetServer::watch_for_changes() was removed. Instead, set it directly on the AssetPlugin.

app
  .add_plugin(DefaultPlugins.set(AssetPlugin {
    watch_for_changes: true,
    ..default()
  }))

Spawn now takes a Bundle #

// Old (0.8):
commands
  .spawn()
  .insert_bundle((A, B, C));
// New (0.9)
commands.spawn((A, B, C));

// Old (0.8):
commands.spawn_bundle((A, B, C));
// New (0.9)
commands.spawn((A, B, C));

// Old (0.8):
let entity = commands.spawn().id();
// New (0.9)
let entity = commands.spawn_empty().id();

// Old (0.8)
let entity = world.spawn().id();
// New (0.9)
let entity = world.spawn_empty().id();

Accept Bundles for insert and remove. Deprecate insert/remove_bundle #

Replace insert_bundle with insert:

// Old (0.8)
commands.spawn().insert_bundle(SomeBundle::default());
// New (0.9)
commands.spawn_empty().insert(SomeBundle::default());

Replace remove_bundle with remove:

// Old (0.8)
commands.entity(some_entity).remove_bundle::<SomeBundle>();
// New (0.9)
commands.entity(some_entity).remove::<SomeBundle>();

Replace remove_bundle_intersection with remove_intersection:

// Old (0.8)
world.entity_mut(some_entity).remove_bundle_intersection::<SomeBundle>();
// New (0.9)
world.entity_mut(some_entity).remove_intersection::<SomeBundle>();

Consider consolidating as many operations as possible to improve ergonomics and cut down on archetype moves:

// Old (0.8)
commands.spawn()
  .insert_bundle(SomeBundle::default())
  .insert(SomeComponent);

// New (0.9) - Option 1
commands.spawn_empty().insert((
  SomeBundle::default(),
  SomeComponent,
))

// New (0.9) - Option 2
commands.spawn((
  SomeBundle::default(),
  SomeComponent,
))

Implement Bundle for Component. Use Bundle tuples for insertion #

The #[bundle] attribute is no longer required when deriving Bundle for nested bundles.

#[derive(Bundle)]
struct PlayerBundle {
    #[bundle] // Remove this line
    sprite_bundle: SpriteBundle,
    collider: Collider,
}

Replace the bool argument of Timer with TimerMode #

    Replace Timer::new(duration, false) with Timer::new(duration, TimerMode::Once).
    Replace Timer::new(duration, true) with Timer::new(duration, TimerMode::Repeating).
    Replace Timer::from_seconds(seconds, false) with Timer::from_seconds(seconds, TimerMode::Once).
    Replace Timer::from_seconds(seconds, true) with Timer::from_seconds(seconds, TimerMode::Repeating).
    Change timer.repeating() to timer.mode() == TimerMode::Repeating.

Add global time scaling #

Some Time methods were renamed for consistency.

The values returned by most methods are now scaled by a value optionally set with set_relative_speed. Most systems should continue to use these scaled values. If you need unscaled time, use the new methods prefixed with raw_.

// Old (Bevy 0.8)
let dur: Duration = time.time_since_startup();
let secs: f32 = time.time_since_startup().as_secs_f32();
let secs: f64 = time.seconds_since_startup();

// New (Bevy 0.9)
let dur: Duration = time.elapsed();
let secs: f32 = time.elapsed_seconds();
let secs: f64 = time.elapsed_seconds_f64();

Change UI coordinate system to have origin at top left corner #

All flex layout should be inverted (ColumnReverse => Column, FlexStart => FlexEnd, WrapReverse => Wrap) System where dealing with cursor position should be changed to account for cursor position being based on the top left instead of bottom left
Rename UiColor to BackgroundColor #

UiColor has been renamed to BackgroundColor. This change affects NodeBundle, ButtonBundle and ImageBundle. In addition, the corresponding field on ExtractedUiNode has been renamed to background_color for consistency.
Make the default background color of NodeBundle transparent #

If you want a NodeBundle with a white background color, you must explicitly specify it:

// Old (Bevy 0.8)
let node = NodeBundle {
    ..default()
}

// New (Bevy 0.9)
let node = NodeBundle {
    background_color: Color::WHITE.into(),
    ..default()
}

Clarify bevy::ui::Node field and documentation #

All references to the old size name has been changed, to access bevy::ui::Node size field use calculated_size
Remove Size and UiRect generics #

The generic T of Size and UiRect got removed and instead they both now always use Val. If you used a Size<f32> consider replacing it with a Vec2 which is way more powerful.
Remove margins.rs #

The Margins type got removed. To migrate you just have to change every occurrence of Margins to UiRect.
Move Size to bevy_ui #

The Size type got moved from bevy::math to bevy::ui. To migrate you just have to import bevy::ui::Size instead of bevy::math::Math or use the bevy::prelude instead.
Move Rect to bevy_ui and rename it to UiRect #

The Rect type got renamed to UiRect. To migrate you just have to change every occurrence of Rect to UiRect.
Move sprite::Rect into bevy_math #

The bevy::sprite::Rect type moved to the math utility crate as bevy::math::Rect. You should change your imports from use bevy::sprite::Rect to use bevy::math::Rect.
Exclusive Systems Now Implement System. Flexible Exclusive System Params #

Calling .exclusive_system() is no longer required (or supported) for converting exclusive system functions to exclusive systems:

// Old (0.8)
app.add_system(some_exclusive_system.exclusive_system());
// New (0.9)
app.add_system(some_exclusive_system);

Converting “normal” parallel systems to exclusive systems is done by calling the exclusive ordering apis:

// Old (0.8)
app.add_system(some_system.exclusive_system().at_end());
// New (0.9)
app.add_system(some_system.at_end());

Query state in exclusive systems can now be cached via ExclusiveSystemParams, which should be preferred for clarity and performance reasons:

// Old (0.8)
fn some_system(world: &mut World) {
  let mut transforms = world.query::<&Transform>();
  for transform in transforms.iter(world) {
  }
}
// New (0.9)
fn some_system(world: &mut World, transforms: &mut QueryState<&Transform>) {
  for transform in transforms.iter(world) {
  }
}

The IntoExclusiveSystem trait was removed. Use IntoSystem instead.

The ExclusiveSystemDescriptorCoercion trait was removed. You can delete any imports of it.
Merge TextureAtlas::from_grid_with_padding into TextureAtlas::from_grid through option arguments #

TextureAtlas::from_grid_with_padding was merged into from_grid which takes two additional parameters for padding and an offset.

// 0.8
TextureAtlas::from_grid(texture_handle, Vec2::new(24.0, 24.0), 7, 1);
// 0.9
TextureAtlas::from_grid(texture_handle, Vec2::new(24.0, 24.0), 7, 1, None, None)

// 0.8
TextureAtlas::from_grid_with_padding(texture_handle, Vec2::new(24.0, 24.0), 7, 1, Vec2::new(4.0, 4.0));
// 0.9
TextureAtlas::from_grid(texture_handle, Vec2::new(24.0, 24.0), 7, 1, Some(Vec2::new(4.0, 4.0)), None)

Rename play to start and add new play method that won't overwrite the existing animation if it's already playing #

If you were using play to restart an animation that was already playing, that functionality has been moved to start. Now, play won’t have any effect if the requested animation is already playing.
Change gamepad.rs tuples to normal structs #

The Gamepad, GamepadButton, GamepadAxis, GamepadEvent and GamepadEventRaw types are now normal structs instead of tuple structs and have a new() function. To migrate change every instantiation to use the new() function instead and use the appropriate field names instead of .0 and .1.
Add getters and setters for InputAxis and ButtonSettings #

AxisSettings now has a new(), which may return an AxisSettingsError. AxisSettings fields made private; now must be accessed through getters and setters. There’s a dead zone, from .deadzone_upperbound() to .deadzone_lowerbound(), and a live zone, from .deadzone_upperbound() to .livezone_upperbound() and from .deadzone_lowerbound() to .livezone_lowerbound(). AxisSettings setters no longer panic. ButtonSettings fields made private; now must be accessed through getters and setters. ButtonSettings now has a new(), which may return a ButtonSettingsError.
Add GamepadInfo, expose gamepad names #

    Pattern matches on GamepadEventType::Connected will need to be updated, as the form of the variant has changed.
    Code that requires GamepadEvent, GamepadEventRaw or GamepadEventType to be Copy will need to be updated.

Gamepad type is Copy; do not require / return references to it in Gamepads API #

    Gamepads::iter now returns an iterator of Gamepad. rather than an iterator of &Gamepad.
    Gamepads::contains now accepts a Gamepad, rather than a &Gamepad.

Update wgpu to 0.14.0, naga to 0.10.0, winit to 0.27.4, raw-window-handle to 0.5.0, ndk to 0.7 #

Adjust usage of bevy_window::WindowDescriptor’s cursor_locked to cursor_grab_mode, and adjust its type from bool to bevy_window::CursorGrabMode.
Support monitor selection for all window modes. #

MonitorSelection was moved out of WindowPosition::Centered, into WindowDescriptor. MonitorSelection::Number was renamed to MonitorSelection::Index.

// Before
.insert_resource(WindowDescriptor {
    position: WindowPosition::Centered(MonitorSelection::Number(1)),
    ..default()
})
// After
.add_plugins(DefaultPlugins.set(WindowPlugin {
    window: WindowDescriptor {
        monitor: MonitorSelection::Index(1),
        position: WindowPosition::Centered,
        ..default()
    },
    ..default()
}))

Window::set_position now takes a MonitorSelection as argument.

window.set_position(MonitorSelection::Current, position);

Rename system chaining to system piping #

The .chain(handler_system) method on systems is now .pipe(handler_system). The IntoChainSystem trait is now IntoPipeSystem, and the ChainSystem struct is now PipeSystem.
Add associated constant IDENTITY to Transform and friends. #

The method identity() on Transform, GlobalTransform and TransformBundle has been removed. Use the associated constant IDENTITY instead.
Rename Transform::mul_vec3 to transform_point and improve docs #

Transform::mul_vec3 has been renamed to transform_point.
Remove Transform::apply_non_uniform_scale #

Transform::apply_non_uniform_scale has been removed. It can be replaced with the following snippet:

transform.scale *= scale_factor;

Remove face_toward.rs #

The FaceToward trait got removed. To migrate you just have to change every occurrence of Mat4::face_toward to Mat4::look_at_rh.
Replace WorldQueryGats trait with actual gats #

Replace usage of WorldQueryGats assoc types with the actual gats on WorldQuery trait
Add a method for accessing the width of a Table #

Any use of Table::len should now be Table::entity_count. Any use of Table::capacity should now be Table::entity_capacity.
Make Handle::<T> field id private, and replace with a getter #

If you were accessing the value handle.id, you can now do so with handle.id()
Add TimeUpdateStrategy resource for manual Time updating #

Changes the value reported by time.delta() on startup.

Before it would be [0, 0, correct] and this PR changes it to be [0, "approximately the time between the time_system and present_frame", correct].
Add methods for silencing system-order ambiguity warnings #

Ambiguity sets have been replaced with a simpler API.

// These systems technically conflict, but we don't care which order they run in.
fn jump_on_click(mouse: Res<Input<MouseButton>>, mut transforms: Query<&mut Transform>) { ... }
fn jump_on_spacebar(keys: Res<Input<KeyCode>>, mut transforms: Query<&mut Transform>) { ... }

// Old (Bevy 0.8)
#[derive(AmbiguitySetLabel)]
struct JumpSystems;

app
  .add_system(jump_on_click.in_ambiguity_set(JumpSystems))
  .add_system(jump_on_spacebar.in_ambiguity_set(JumpSystems));

// New (Bevy 0.9)
app
  .add_system(jump_on_click.ambiguous_with(jump_on_spacebar))
  .add_system(jump_on_spacebar);

Remove unused DepthCalculation enum #

Remove references to bevy_render::camera::DepthCalculation, such as use bevy_render::camera::DepthCalculation. Remove depth_calculation fields from Projections.
Make raw_window_handle field in Window and ExtractedWindow an Option. #

Window::raw_window_handle() now returns Option<RawWindowHandleWrapper>.
Fix inconsistent children removal behavior #

    Queries with Changed<Children> will no longer match entities that had all of their children removed using remove_children.
    RemovedComponents<Children> will now contain entities that had all of their children removed using remove_children.

Entity's “ID” should be named “index” instead #

The Entity::id() method was renamed to Entity::index().
Remove ExactSizeIterator from QueryCombinationIter #

len is no longer implemented for QueryCombinationIter. You can get the same value with size_hint().0, but be aware that values exceeding usize::MAX will be returned as usize::MAX.
Query filter types must be ReadOnlyWorldQuery #

Query filter (F) generics are now bound by ReadOnlyWorldQuery, rather than WorldQuery. If for some reason you were requesting Query<&A, &mut B>, please use Query<&A, With<B>> instead.
Add pop method for List trait. #

Any custom type that implements the List trait will now need to implement the pop method.
Remove an outdated workaround for impl Trait #

The methods Schedule::get_stage and get_stage_mut now accept impl StageLabel instead of &dyn StageLabel.
Add a change detection bypass and manual control over change ticks #

Add the Inner associated type and new methods to any type that you’ve implemented DetectChanges for.
Make internal struct ShaderData non-pub #

Removed ShaderData from the public API, which was only ever used internally. No public function was using it so there should be no need for any migration action.
Make Children constructor pub(crate). #

Children::with() is now renamed Children::from_entities() and is now pub(crate)
Rename Handle::as_weak() to cast_weak() #

    Rename Handle::as_weak uses to Handle::cast_weak

The method now properly sets the associated type uuid if the handle is a direct reference (e.g. not a reference to an AssetPath), so adjust you code accordingly if you relied on the previous behavior.
Remove Sync bound from Local #

Any code relying on Local<T> having T: Resource may have to be changed, but this is unlikely.
Add FromWorld bound to T in Local<T> #

It might be possible for references to Locals without T: FromWorld to exist, but these should be exceedingly rare and probably dead code. In the event that one of these is encountered, the easiest solutions are to delete the code or wrap the inner T in an Option to allow it to be default constructed to None.

This may also have other smaller implications (such as Debug representation), but serialization is probably the most prominent.
Swap out num_cpus for std::thread::available_parallelism #

bevy_tasks::logical_core_count and bevy_tasks::physical_core_count have been removed. logical_core_count has been replaced with bevy_tasks::available_parallelism, which works identically. If bevy_tasks::physical_core_count is required, the num_cpus crate can be used directly, as these two were just aliases for num_cpus APIs.
Changed diagnostics from seconds to milliseconds #

Diagnostics values are now in milliseconds. If you need seconds, simply divide it by 1000.0;
Add Exponential Moving Average into diagnostics #

LogDiagnosticsPlugin now records the smoothed value rather than the raw value.

    For diagnostics recorded less often than every 0.1 seconds, this change to defaults will have no visible effect.
    For discrete diagnostics where this smoothing is not desirable, set a smoothing factor of 0 to disable smoothing.
    The average of the recent history is still shown when available.

Nested spawns on scope #

If you were using explicit lifetimes and Passing Scope you’ll need to specify two lifetimes now.

// 0.8
fn scoped_function<'scope>(scope: &mut Scope<'scope, ()>) {}

// 0.9
fn scoped_function<'scope>(scope: &Scope<'_, 'scope, ()>) {}

scope.spawn_local changed to scope.spawn_on_scope this should cover cases where you needed to run tasks on the local thread, but does not cover spawning Nonsend Futures. Spawning of NonSend futures on scope is no longer supported.
Extract Resources into their own dedicated storage #

Resources have been moved to Resources under Storages in World. All code dependent on Archetype::unique_components(_mut) should access it via world.storages().resources() instead.

All APIs accessing the raw data of individual resources (mutable and read-only) have been removed as these APIs allowed for unsound unsafe code. All usages of these APIs should be changed to use World::{get, insert, remove}_resource.
Clean up Fetch code #

Changed: Fetch::table_fetch and Fetch::archetype_fetch have been merged into a single Fetch::fetch function.
Rename ElementState to ButtonState #

The ElementState type received a rename and is now called ButtonState. To migrate you just have to change every occurrence of ElementState to ButtonState.
Fix incorrect and unnecessary normal-mapping code #

prepare_normal from the bevy_pbr::pbr_functions shader import has been reworked.

Before:

    pbr_input.world_normal = in.world_normal;

    pbr_input.N = prepare_normal(
        pbr_input.material.flags,
        in.world_normal,
#ifdef VERTEX_TANGENTS
#ifdef STANDARDMATERIAL_NORMAL_MAP
        in.world_tangent,
#endif
#endif
        in.uv,
        in.is_front,
    );

After:

    pbr_input.world_normal = prepare_world_normal(
        in.world_normal,
        (material.flags & STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0u,
        in.is_front,
    );

    pbr_input.N = apply_normal_mapping(
        pbr_input.material.flags,
        pbr_input.world_normal,
#ifdef VERTEX_TANGENTS
#ifdef STANDARDMATERIAL_NORMAL_MAP
        in.world_tangent,
#endif
#endif
        in.uv,
    );

Scene serialization format improvements from #6354, #6345, and #5723 #

    The root of the scene is now a struct rather than a list
    Components are now a map keyed by type name rather than a list
    Type information is now omitted when possible, making scenes much more compact

Scenes serialized with Bevy 0.8 will need to be recreated, but it is possible to hand-edit scenes to match the new format.

Here's an example scene in the old and new format:

// Old (Bevy 0.8)
[
  (
    entity: 0,
    components: [
      {
        "type": "bevy_transform::components::transform::Transform",
        "struct": {
          "translation": {
            "type": "glam::vec3::Vec3",
            "value": (0.0, 0.0, 0.0),
          },
          "rotation": {
            "type": "glam::quat::Quat",
            "value": (0.0, 0.0, 0.0, 1.0),
          },
          "scale": {
            "type": "glam::vec3::Vec3",
            "value": (1.0, 1.0, 1.0),
          },
        },
      },
      {
        "type": "scene::ComponentB",
        "struct": {
          "value": {
            "type": "alloc::string::String",
            "value": "hello",
          },
        },
      },
      {
        "type": "scene::ComponentA",
        "struct": {
          "x": {
            "type": "f32",
            "value": 1.0,
          },
          "y": {
            "type": "f32",
            "value": 2.0,
          },
        },
      },
    ],
  ),
  (
    entity: 1,
    components: [
      {
        "type": "scene::ComponentA",
        "struct": {
          "x": {
            "type": "f32",
            "value": 3.0,
          },
          "y": {
            "type": "f32",
            "value": 4.0,
          },
        },
      },
    ],
  ),
]

// New (Bevy 0.9)
(
  entities: {
    0: (
      components: {
        "bevy_transform::components::transform::Transform": (
          translation: (
            x: 0.0,
            y: 0.0,
            z: 0.0
          ),
          rotation: (0.0, 0.0, 0.0, 1.0),
          scale: (
            x: 1.0,
            y: 1.0,
            z: 1.0
          ),
        ),
        "scene::ComponentB": (
          value: "hello",
        ),
        "scene::ComponentA": (
          x: 1.0,
          y: 2.0,
        ),
      },
    ),
    1: (
      components: {
        "scene::ComponentA": (
          x: 3.0,
          y: 4.0,
        ),
      },
    ),
  }
)

Derive Reflect + FromReflect for input types #

    Input<T> now implements Reflect via #[reflect] instead of #[reflect_value]. This means it now exposes its private fields via the Reflect trait rather than being treated as a value type. For code that relies on the Input<T> struct being treated as a value type by reflection, it is still possible to wrap the Input<T> type with a wrapper struct and apply #[reflect_value] to it.
    As a reminder, private fields exposed via reflection are not subject to any stability guarantees.

Relax bounds on Option<T> #

If using Option<T> with Bevy’s reflection API, T now needs to implement FromReflect rather than just Clone. This can be achieved easily by simply deriving FromReflect:


// OLD
#[derive(Reflect, Clone)]
struct Foo;

let reflected: Box<dyn Reflect> = Box::new(Some(Foo));

// NEW
#[derive(Reflect, FromReflect)]
struct Foo;

let reflected: Box<dyn Reflect> = Box::new(Some(Foo));

Note: You can still derive Clone, but it’s not required in order to compile.
Remove ReflectMut in favor of Mut<dyn Reflect> #

    relax T: ?Sized bound in Mut<T>
    replace all instances of ReflectMut with Mut<dyn Reflect>

remove blanket Serialize + Deserialize requirement for Reflect on generic types #

.register_type for generic types like Option<T>, Vec<T>, HashMap<K, V> will no longer insert ReflectSerialize and ReflectDeserialize type data. Instead you need to register it separately for concrete generic types like so:

  .register_type::<Option<String>>()
  .register_type_data::<Option<String>, ReflectSerialize>()
  .register_type_data::<Option<String>, ReflectDeserialize>()

Utility methods for Val #

It is no longer possible to use the +, +=, -, or -= operators with Val or Size.

Use the new try_add and try_sub methods instead and perform operations on Size's height and width fields separately.
Allow passing glam vector types as vertex attributes #

Implementations of From<Vec<[u16; 4]>> and From<Vec<[u8; 4]>> for VertexAttributeValues have been removed. I you're passing either Vec<[u16; 4]> or Vec<[u8; 4]> into Mesh::insert_attribute it will now require wrapping it with right the VertexAttributeValues enum variant.

Migration Guide: 0.9 to 0.10

Bevy relies heavily on improvements in the Rust language and compiler. As a result, the Minimum Supported Rust Version (MSRV) is "the latest stable release" of Rust.
Migrate engine to Schedule v3 (stageless) #
Rendering
ECS

    Calls to .label(MyLabel) should be replaced with .in_set(MySet)

    SystemLabel derives should be replaced with SystemSet. You will also need to add the Debug, PartialEq, Eq, and Hash traits to satisfy the new trait bounds.

    Stages have been removed. Replace these with system sets, and then add command flushes using the apply_system_buffers exclusive system where needed.

    The CoreStage, StartupStage, RenderStage, and AssetStage enums have been replaced with CoreSet, StartupSet, RenderSet and AssetSet. The same scheduling guarantees have been preserved.

    with_run_criteria has been renamed to run_if. Run criteria have been renamed to run conditions for clarity, and should now simply return a bool instead of schedule::ShouldRun.

    Looping run criteria and state stacks have been removed. Use an exclusive system that runs a schedule if you need this level of control over system control flow.

    App::add_state now takes 0 arguments: the starting state is set based on the Default impl.

    Instead of creating SystemSet containers for systems that run in stages, use my_system.in_schedule(OnEnter(State::Variant)) or its OnExit sibling.

    For app-level control flow over which schedules get run when (such as for rollback networking), create your own schedule and insert it under the CoreSchedule::Outer label.

    Fixed timesteps are now evaluated in a schedule, rather than controlled via run criteria. The run_fixed_timestep system runs this schedule between CoreSet::First and CoreSet::PreUpdate by default.

    Command flush points introduced by AssetStage have been removed. If you were relying on these, add them back manually.

    The calculate_bounds system, with the CalculateBounds label, is now in CoreSet::Update, rather than in CoreSet::PostUpdate before commands are applied. You may need to order your movement systems to occur before this system in order to avoid system order ambiguities in culling behavior.

    The RenderLabel AppLabel was renamed to RenderApp for clarity

    When testing systems or otherwise running them in a headless fashion, simply construct and run a schedule using Schedule::new() and World::run_schedule rather than constructing stages

    States have been dramatically simplified: there is no longer a “state stack”. To queue a transition to the next state, call NextState::set

    Strings can no longer be used as a SystemLabel or SystemSet. Use a type, or use the system function instead.

Stages #

Stages had two key elements: they ran one after another, and they applied commands at their end.

The former can be replaced by system sets (unless you need branching or looping scheduling logic, in which case you should use a schedule), and the latter can be controlled manually via apply_system_buffers.

To migrate from Bevy's built-in stages, we've provided the CoreSet, StartupSet and RenderSet system sets. Command flushes have already been added to these, but if you have added custom stages you may need to add your own if you were relying on that behavior.

Before:

app
    .add_system_to_stage(CoreStage::PostUpdate, my_system)
    .add_startup_system_to_stage(StartupStage::PostStartup, my_startup_system);

After:

app
    .add_system(my_system.in_base_set(CoreSet::PostUpdate))
    .add_startup_system(my_startup_system.in_base_set(StartupSet::PostStartup));

If you had your own stage:

// Bevy 0.9
#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
pub struct AfterUpdate;

app.add_stage_after(CoreStage::Update, AfterUpdate, SystemStage::parallel());

// Bevy 0.10, no command flush
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
#[system_set(base)]
pub struct AfterUpdate;

app.configure_set(
    AfterUpdate
        .after(CoreSet::UpdateFlush)
        .before(CoreSet::PostUpdate),
);

// Bevy 0.10, with a command flush
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
#[system_set(base)]
pub enum AfterUpdate {
    Parallel,
    CommandFlush
}

app.configure_sets(
    (
        CoreSet::UpdateFlush,
        AfterUpdate::Parallel,
        AfterUpdate::CommandFlush,
        CoreSet::PostUpdate,
    ).chain()
).add_system(apply_system_buffers.in_base_set(AfterUpdate::CommandFlush));

Label types #

System labels have been renamed to systems sets and unified with stage labels. The StageLabel trait should be replaced by a system set, using the SystemSet trait as dicussed immediately below.

Before:

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
enum MyStage {
    BeforeRound,
    AfterRound,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
enum MySystem {
    ComputeForces,
    FindCollisions,
}

After:

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
enum MySet {
    BeforeRound,
    AfterRound,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
enum MySystem {
    ComputeForces,
    FindCollisions,
}

System sets (Bevy 0.9) #

In Bevy 0.9, you could use the SystemSet type and various methods to configure many systems at once. 
Additionally, this was the only way to interact with various scheduling APIs like run criteria.

Before:

app.add_system_set(SystemSet::new().with_system(a).with_system(b).with_run_criteria(my_run_criteria));

After:

app.add_systems((a, b).run_if(my_run_condition));

Ambiguity detection #

The ReportExecutionOrderAmbiguities resource has been removed. Instead, this is configured on a per-schedule basis.

app.edit_schedule(CoreSchedule::Main, |schedule| {
    schedule.set_build_settings(ScheduleBuildSettings {
        ambiguity_detection: LogLevel::Warn,
        ..default()
    });
})

Fixed timesteps #

The FixedTimestep run criteria has been removed, and is now handled by either a schedule or the on_timer / on_fixed_timer run conditions.

Before:

app.add_stage_after(
    CoreStage::Update,
    FixedUpdateStage,
    SystemStage::parallel()
        .with_run_criteria(
            FixedTimestep::step(0.5)
        )
        .with_system(fixed_update),
);

After:

// This will affect the update frequency of fixed time for your entire app
app.insert_resource(FixedTime::new_from_secs(0.5))

    // This schedule is automatically added with DefaultPlugins
    .add_system(fixed_update.in_schedule(CoreSchedule::FixedUpdate));

Apps may now only have one unified fixed timestep. CoreSchedule::FixedTimestep is intended to be used for determinism and stability during networks, physics and game mechanics. Unlike timers, it will run repeatedly if more than a single period of time has elapsed since it was last run.

It is not intended to serve as a looping timer to regularly perform work or poll. If you were relying on multiple FixedTimestep run criteria with distinct periods, you should swap to using timers, via the on_timer(MY_PERIOD) or on_fixed_timer(MY_PERIOD) run conditions.

Before:

app.add_system_set(
    SystemSet::new()
        .with_run_criteria(FixedTimestep::step(0.5))
        .with_system(update_pathfinding),
)
.add_system_set(
    SystemSet::new()
        .with_run_criteria(FixedTimestep::step(0.1))
        .with_system(apply_damage_over_time),
);

After:

app
.add_system(update_pathfinding.run_if(on_timer(Duration::from_secs_f32(0.5))))
.add_system(apply_damage_over_time.run_if(on_timer(Duration::from_secs_f32(0.1))));

States #

States have been significantly simplied and no longer have a state stack. Each state type (usually an enum), requires the States trait, typically implemented via the derive macro.

For example:

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    #[default]
    Menu,
    InGame,
}

App::add_state no longer takes an argument: the starting state is now controlled via the Default impl for your state type.

To access the current state of the the States type above, use Res<State<AppState>, and access the tuple field via .0. To queue up a state transition, use ResMut<NextState<AppState>> and call .set(AppState::Menu).

State transitions are now applied via the apply_state_transitions exclusive system, a copy of which is added CoreSet::StateTransitions when you call App::add_state. You can add more copies as needed, specific to the state being applied.

OnEnter and OnExit systems now live in schedules, run on the World via the apply_state_transitions system. By contrast, OnUpdate is now a system set which is nested within CoreSet::Update.

Before:

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
enum AppState {
    Menu,
    InGame,
}

app.add_state(AppState::Menu)
    .add_system_set(SystemSet::on_enter(AppState::Menu).with_system(setup_menu))
    .add_system_set(SystemSet::on_update(AppState::Menu).with_system(menu))
    .add_system_set(SystemSet::on_exit(AppState::Menu).with_system(cleanup_menu))

After:

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    #[default]
    Menu,
    InGame,
}

app.add_state::<AppState>()
    .add_system(setup_menu.in_schedule(OnEnter(AppState::Menu)))
    .add_system(menu.in_set(OnUpdate(AppState::Menu)))
    .add_system(cleanup_menu.in_schedule(OnExit(AppState::Menu)));

When you need to run your state-specific systems outside of CoreSet::Update, you can use the built-in in_state run condition.
Windows as Entities #
Windowing

Replace WindowDescriptor with Window.

Change width and height fields into a WindowResolution, either by doing

WindowResolution::new(width, height) // Explicitly
// or using From<_> for tuples for convenience
(1920., 1080.).into()

Replace any WindowCommand code to just modify the Window’s fields directly and creating/closing windows is now by spawning/despawning an entity with a Window component like so:

let window = commands.spawn(Window { ... }).id(); // open window
commands.entity(window).despawn(); // close window

To get a window, you now need to use a Query instead of a Res

// 0.9
fn count_pixels(windows: Res<Windows>) {
    let Some(primary) = windows.get_primary() else {
        return;
    };
    println!("{}", primary.width() * primary.height());
}

// 0.10
fn count_pixels(primary_query: Query<&Window, With<PrimaryWindow>>) {
    let Ok(primary) = primary_query.get_single() else {
        return;
    };
    println!("{}", primary.width() * primary.height());
}

Make the SystemParam derive macro more flexible #
ECS

The lifetime 's has been removed from EventWriter. Any code that explicitly specified the lifetimes for this type will need to be updated.

// 0.9
#[derive(SystemParam)]
struct MessageWriter<'w, 's> {
    events: EventWriter<'w, 's, Message>,
}

// 0.10
#[derive(SystemParam)]
struct MessageWriter<'w> {
    events: EventWriter<'w, Message>,
}

Basic adaptive batching for parallel query iteration #
ECS

The batch_size parameter for Query(State)::par_for_each(_mut) has been removed. These calls will automatically compute a batch size for you. Remove these parameters from all calls to these functions.

// 0.9
fn parallel_system(query: Query<&MyComponent>) {
   query.par_for_each(32, |comp| {
        ...
   });
}

// 0.10
fn parallel_system(query: Query<&MyComponent>) {
   query.par_iter().for_each(|comp| {
        ...
   });
}

Enum Visibility component #
Rendering

    Evaluation of the visibility.is_visible field should now check for visibility == Visibility::Inherited.
    Setting the visibility.is_visible field should now directly set the value: *visibility = Visibility::Inherited.
    Usage of Visibility::VISIBLE or Visibility::INVISIBLE should now use Visibility::Inherited or Visibility::Hidden respectively.
    ComputedVisibility::INVISIBLE and SpatialBundle::VISIBLE_IDENTITY have been renamed to ComputedVisibility::HIDDEN and SpatialBundle::INHERITED_IDENTITY respectively.

bevy_reflect: Pre-parsed paths #
Animation
Reflection

GetPath methods have been renamed according to the following:

    path -> reflect_path
    path_mut -> reflect_path_mut
    get_path -> path
    get_path_mut -> path_mut

Remove App::add_sub_app #
App

App::add_sub_app has been removed in favor of App::insert_sub_app. Use SubApp::new and insert it via App::insert_sub_app

// 0.9
let mut sub_app = App::new()
// Build subapp here
app.add_sub_app(MySubAppLabel, sub_app, extract_fn);

// 0.10
let mut sub_app = App::new()
// Build subapp here
app.insert_sub_app(MySubAppLabel, SubApp::new(sub_app, extract_fn));

Make HandleUntyped::id private #
Assets

Instead of directly accessing the ID of a HandleUntyped as handle.id, use the new getter handle.id().
Break CorePlugin into TaskPoolPlugin, TypeRegistrationPlugin, FrameCountPlugin. #
Core

CorePlugin was broken into separate plugins. If not using DefaultPlugins or MinimalPlugins PluginGroups, the replacement for CorePlugin is now to add TaskPoolPlugin, TypeRegistrationPlugin, and FrameCountPlugin to the app.
Immutable sparse sets for metadata storage #
ECS

Table::component_capacity() has been removed as Tables do not support adding/removing columns after construction.
Split Component Ticks #
ECS

Various low level APIs interacting with the change detection ticks no longer return &UnsafeCell<ComponentTicks>, instead returning TickCells which contains two separate &UnsafeCell<Tick>s instead.

// 0.9
column.get_ticks(row).deref().changed

// 0.10
column.get_ticks(row).changed.deref()

Document and lock down types in bevy_ecs::archetype #
ECS

ArchetypeId, ArchetypeGeneration, and ArchetypeComponentId are all now opaque IDs and cannot be turned into a numeric value. Please file an issue if this does not work for your use case or check bevy_ecs is excessively public for more info.

Archetype and Archetypes are not constructible outside of bevy_ecs now. Use World::archetypes to get a read-only reference to either of these types.
Lock down access to Entities #
ECS

Entities’s Default implementation has been removed. You can fetch a reference to a World’s Entities via World::entities and World::entities_mut.

Entities::alloc_at_without_replacement and AllocAtWithoutReplacement has been made private due to difficulty in using it properly outside of bevy_ecs. If you still need use of this API, please file an issue or check bevy_ecs is excessively public for more info.
Borrow instead of consuming in EventReader::clear #
ECS

EventReader::clear now takes a mutable reference instead of consuming the event reader. This means that clear now needs explicit mutable access to the reader variable, which previously could have been omitted in some cases:

// Old (0.9)
fn clear_events(reader: EventReader<SomeEvent>) {
  reader.clear();
}

// New (0.10)
fn clear_events(mut reader: EventReader<SomeEvent>) {
  reader.clear();
}

Newtype ArchetypeRow and TableRow #
ECS

Archetype indices and Table rows have been newtyped as ArchetypeRow and TableRow.
Round out the untyped api s #
ECS

MutUntyped::into_inner now marks things as changed.
Extend EntityLocation with TableId and TableRow #
ECS

A World can only hold a maximum of 232 - 1 archetypes and tables now. If your use case requires more than this, please file an issue explaining your use case.
Remove ExclusiveSystemParam::apply #
ECS

The trait method ExclusiveSystemParamState::apply has been removed. If you have an exclusive system with buffers that must be applied, you should apply them within the body of the exclusive system.
Remove the SystemParamState trait and remove types like ResState #
ECS

The traits SystemParamState and SystemParamFetch have been removed, and their functionality has been transferred to SystemParam.

The trait ReadOnlySystemParamFetch has been replaced with ReadOnlySystemParam.

// 0.9
impl SystemParam for MyParam<'_, '_> {
    type State = MyParamState;
}
unsafe impl SystemParamState for MyParamState {
    fn init(world: &mut World, system_meta: &mut SystemMeta) -> Self { ... }
}
unsafe impl<'w, 's> SystemParamFetch<'w, 's> for MyParamState {
    type Item = MyParam<'w, 's>;
    fn get_param(&mut self, ...) -> Self::Item;
}
unsafe impl ReadOnlySystemParamFetch for MyParamState { }

// 0.10
unsafe impl SystemParam for MyParam<'_, '_> {
    type State = MyParamState;
    type Item<'w, 's> = MyParam<'w, 's>;
    fn init_state(world: &mut World, system_meta: &mut SystemMeta) -> Self::State { ... }
    fn get_param<'w, 's>(state: &mut Self::State, ...) -> Self::Item<'w, 's>;
}
unsafe impl ReadOnlySystemParam for MyParam<'_, '_> { }

Panic on dropping NonSend in non-origin thread. #
ECS

Normal resources and NonSend resources no longer share the same backing storage. If R: Resource, then NonSend<R> and Res<R> will return different instances from each other. If you are using both Res<T> and NonSend<T> (or their mutable variants), to fetch the same resources, it’s strongly advised to use Res<T>.
Document alignment requirements of Ptr, PtrMut and OwningPtr #
ECS

Safety invariants on bevy_ptr types’ new byte_add and byte_offset methods have been changed. All callers should re-audit for soundness.
Added resource_id and changed init_resource and init_non_send_resource to return ComponentId #
ECS

    Added Components::resource_id.
    Changed World::init_resource to return the generated ComponentId.
    Changed World::init_non_send_resource to return the generated ComponentId.

Replace RemovedComponents<T> backing with Events<Entity> #
ECS

    Add a mut for removed: RemovedComponents<T> since we are now modifying an event reader internally.
    Iterating over removed components now requires &mut removed_components or removed_components.iter() instead of &removed_components.

Remove broken DoubleEndedIterator impls on event iterators #
ECS

ManualEventIterator and ManualEventIteratorWithId are no longer DoubleEndedIterators since the impls didn't work correctly, and any code using this was likely broken.
Rename Tick::is_older_than to Tick::is_newer_than #
ECS

Replace usages of Tick::is_older_than with Tick::is_newer_than.
Cleanup system sets called labels #
ECS

PrepareAssetLabel is now called PrepareAssetSet
Simplify generics for the SystemParamFunction trait #
ECS

For the SystemParamFunction trait, the type parameters In, Out, and Param have been turned into associated types.

// 0.9
fn my_generic_system<T, In, Out, Param, Marker>(system_function: T)
where
    T: SystemParamFunction<In, Out, Param, Marker>,
    Param: SystemParam,
{ ... }

// 0.10
fn my_generic_system<T, Marker>(system_function: T)
where
    T: SystemParamFunction<Marker>,
{ ... }

For the ExclusiveSystemParamFunction trait, the type parameter Param has been turned into an associated type. Also, In and Out associated types have been added, since exclusive systems now support system piping.

// 0.9
fn my_exclusive_system<T, Param, Marker>(system_function: T)
where
    T: ExclusiveSystemParamFunction<Param, Marker>,
    T: Param: ExclusiveSystemParam,
{ ... }

// 0.10
fn my_exclusive_system<T, Marker>(system_function: T)
where
    T: ExclusiveSystemParamFunction<Marker>,
{ ... }

Deprecate ChangeTrackers<T> in favor of Ref<T> #
ECS

ChangeTrackers<T> has been deprecated, and will be removed in the next release. Any usage should be replaced with Ref<T>.

// 0.9
fn my_system(q: Query<(&MyComponent, ChangeTrackers<MyComponent>)>) {
    for (value, trackers) in &q {
        if trackers.is_changed() {
            // Do something with `value`.
        }
    }
}

// 0.10
fn my_system(q: Query<Ref<MyComponent>>) {
    for value in &q {
        if value.is_changed() {
            // Do something with `value`.
        }
    }
}

EntityMut: rename remove_intersection to remove and remove to take #
ECS

// 0.9
fn clear_children(parent: Entity, world: &mut World) {
    if let Some(children) = world.entity_mut(parent).remove::<Children>() {
        for &child in &children.0 {
            world.entity_mut(child).remove_intersection::<Parent>();
        }
    }
}

// 0.10
fn clear_children(parent: Entity, world: &mut World) {
    if let Some(children) = world.entity_mut(parent).take::<Children>() {
        for &child in &children.0 {
            world.entity_mut(child).remove::<Parent>();
        }
    }
}

bevy_ecs: ReflectComponentFns without World #
ECS
Reflection

Call World::entity before calling into the changed ReflectComponent methods, most likely user already has a EntityRef or EntityMut which was being queried redundantly.
Allow iterating over with EntityRef over the entire World #
ECS
Scenes

World::iter_entities now returns an iterator of EntityRef instead of Entity. To get the actual ID, use EntityRef::id from the returned EntityRefs.
Remove BuildWorldChildren impl from WorldChildBuilder #
Hierarchy

Hierarchy editing methods such as with_children and push_children have been removed from WorldChildBuilder. You can edit the hierarchy via EntityMut instead.
Rename dynamic feature #
Meta

dynamic feature was renamed to dynamic_linking
bevy_reflect: add insert and remove methods to List #
Reflection

Manual implementors of List need to implement the new methods insert and remove and consider whether to use the new default implementation of push and pop.
bevy_reflect: Decouple List and Array traits #
Reflection

The List trait is no longer dependent on Array. Implementors of List can remove the Array impl and move its methods into the List impl (with only a couple tweaks).

// 0.9
impl Array for Foo {
  fn get(&self, index: usize) -> Option<&dyn Reflect> {/* ... */}
  fn get_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {/* ... */}
  fn len(&self) -> usize {/* ... */}
  fn is_empty(&self) -> bool {/* ... */}
  fn iter(&self) -> ArrayIter {/* ... */}
  fn drain(self: Box<Self>) -> Vec<Box<dyn Reflect>> {/* ... */}
  fn clone_dynamic(&self) -> DynamicArray {/* ... */}
}

impl List for Foo {
  fn insert(&mut self, index: usize, element: Box<dyn Reflect>) {/* ... */}
  fn remove(&mut self, index: usize) -> Box<dyn Reflect> {/* ... */}
  fn push(&mut self, value: Box<dyn Reflect>) {/* ... */}
  fn pop(&mut self) -> Option<Box<dyn Reflect>> {/* ... */}
  fn clone_dynamic(&self) -> DynamicList {/* ... */}
}

// 0.10
impl List for Foo {
  fn get(&self, index: usize) -> Option<&dyn Reflect> {/* ... */}
  fn get_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {/* ... */}
  fn insert(&mut self, index: usize, element: Box<dyn Reflect>) {/* ... */}
  fn remove(&mut self, index: usize) -> Box<dyn Reflect> {/* ... */}
  fn push(&mut self, value: Box<dyn Reflect>) {/* ... */}
  fn pop(&mut self) -> Option<Box<dyn Reflect>> {/* ... */}
  fn len(&self) -> usize {/* ... */}
  fn is_empty(&self) -> bool {/* ... */}
  fn iter(&self) -> ListIter {/* ... */}
  fn drain(self: Box<Self>) -> Vec<Box<dyn Reflect>> {/* ... */}
  fn clone_dynamic(&self) -> DynamicList {/* ... */}
}

Some other small tweaks that will need to be made include:

    Use ListIter for List::iter instead of ArrayIter (the return type from Array::iter)
    Replace array_hash with list_hash in Reflect::reflect_hash for implementors of List

bevy_reflect: Remove ReflectSerialize and ReflectDeserialize registrations from most glam types #
Reflection
Scenes

This PR removes ReflectSerialize and ReflectDeserialize registrations from most glam types. This means any code relying on either of those type data existing for those glam types will need to not do that.

This also means that some serialized glam types will need to be updated. For example, here is Affine3A:

// 0.9
(
  "glam::f32::affine3a::Affine3A": (1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0),

// 0.10
  "glam::f32::affine3a::Affine3A": (
    matrix3: (
      x_axis: (
        x: 1.0,
        y: 0.0,
        z: 0.0,
      ),
      y_axis: (
        x: 0.0,
        y: 1.0,
        z: 0.0,
      ),
      z_axis: (
        x: 0.0,
        y: 0.0,
        z: 1.0,
      ),
    ),
    translation: (
      x: 0.0,
      y: 0.0,
      z: 0.0,
    ),
  )
)

Add AutoMax next to ScalingMode::AutoMin #
Rendering

Rename ScalingMode::Auto to ScalingMode::AutoMin.
Change From<Icosphere> to TryFrom<Icosphere> #
Rendering

// 0.9
shape::Icosphere {
    radius: 0.5,
    subdivisions: 5,
}
.into()

// 0.10
shape::Icosphere {
    radius: 0.5,
    subdivisions: 5,
}
.try_into()
.unwrap()

Add try_* to add_slot_edge, add_node_edge #
Rendering

Remove .unwrap() from add_node_edge and add_slot_edge. For cases where the error was handled, use try_add_node_edge and try_add_slot_edge instead.

Remove .unwrap() from input_node. For cases where the option was handled, use get_input_node instead.
Shader defs can now have a value #
Rendering

    Replace shader_defs.push(String::from("NAME")); by shader_defs.push("NAME".into());
    If you used shader def NO_STORAGE_BUFFERS_SUPPORT, check how AVAILABLE_STORAGE_BUFFER_BINDINGS is now used in Bevy default shaders

Get pixel size from wgpu #
Rendering

PixelInfo has been removed. PixelInfo::components is equivalent to texture_format.describe().components. PixelInfo::type_size can be gotten from texture_format.describe().block_size/ texture_format.describe().components. But note this can yield incorrect results for some texture types like Rg11b10Float.
Run clear trackers on render world #
Rendering

The call to clear_trackers in App has been moved from the schedule to App::update for the main world and calls to clear_trackers have been added for sub_apps in the same function. This was due to needing stronger guarantees. If clear_trackers isn’t called on a world it can lead to memory leaks in RemovedComponents. If you were ordering systems with clear_trackers this is no longer possible.
Rename camera "priority" to "order" #
Rendering

Rename priority to order in usage of Camera.
Reduce branching in TrackedRenderPass #
Rendering

TrackedRenderPass now requires a RenderDevice to construct. To make this easier, use RenderContext.begin_tracked_render_pass instead.

// 0.9
TrackedRenderPass::new(render_context.command_encoder.begin_render_pass(
  &RenderPassDescriptor {
    ...
  },
));

// 0.10
render_context.begin_tracked_render_pass(RenderPassDescriptor {
  ...
});

Make PipelineCache internally mutable. #
Rendering

Most usages of resource_mut::<PipelineCache> and ResMut<PipelineCache> can be changed to resource::<PipelineCache> and Res<PipelineCache> as long as they don’t use any methods requiring mutability - the only public method requiring it is process_queue.
Changed Msaa to Enum #
Rendering

// 0.9
let multi = Msaa { samples: 4 }
// 0.10
let multi = Msaa::Sample4

// 0.9
multi.samples
// 0.10
multi.samples()

Support recording multiple CommandBuffers in RenderContext #
Rendering

RenderContext’s fields are now private. Use the accessors on RenderContext instead, and construct it with RenderContext::new.
Improve OrthographicCamera consistency and usability #
Rendering

    Change window_origin to viewport_origin; replace WindowOrigin::Center with Vec2::new(0.5, 0.5) and WindowOrigin::BottomLeft with Vec2::new(0.0, 0.0)

    For shadow projections and such, replace left, right, bottom, and top with area: Rect::new(left, bottom, right, top)

    For camera projections, remove l/r/b/t values from OrthographicProjection instantiations, as they no longer have any effect in any ScalingMode

    Change ScalingMode::None to ScalingMode::Fixed
        Replace manual changes of l/r/b/t with:
            Arguments in ScalingMode::Fixed to specify size
            viewport_origin to specify offset

    Change ScalingMode::WindowSize to ScalingMode::WindowSize(1.0)

Changed &mut PipelineCache to &PipelineCache #
Rendering

SpecializedComputePipelines::specialize now takes a &PipelineCache instead of a &mut PipelineCache
Introduce detailed_trace macro, use in TrackedRenderPass #
Rendering

Some detailed bevy trace events now require the use of the cargo feature detailed_trace in addition to enabling TRACE level logging to view. Should you wish to see these logs, please compile your code with the bevy feature detailed_trace. Currently, the only logs that are affected are the renderer logs pertaining to TrackedRenderPass functions
Added subdivisions to shape::Plane #
Rendering

shape::Plane now takes an additional subdivisions parameter so users should provide it or use the new shape::Plane::from_size().
Change standard material defaults and update docs #
Rendering

StandardMaterial’s default have now changed to be a fully dielectric material with medium roughness. If you want to use the old defaults, you can set perceptual_roughness = 0.089 and metallic = 0.01 (though metallic should generally only be set to 0.0 or 1.0).
Remove dead code after #7784 #
Rendering

Removed SetShadowViewBindGroup, queue_shadow_view_bind_group(), and LightMeta::shadow_view_bind_group in favor of reusing the prepass view bind group.
Directly extract joints into SkinnedMeshJoints #
Rendering
Animation

ExtractedJoints has been removed. Read the bound bones from SkinnedMeshJoints instead.
Intepret glTF colors as linear instead of sRGB #
Rendering
Assets

No api changes are required, but it's possible that your gltf meshes look different
Send emissive color to uniform as linear instead of sRGB #

    If you have previously manually specified emissive values with Color::rgb() and would like to retain the old visual results, you must now use Color::rgb_linear() instead;
    If you have previously manually specified emissive values with Color::rgb_linear() and would like to retain the old visual results, you'll need to apply a one-time gamma calculation to your channels manually to get the actual linear RGB value:
        For channel values greater than 0.0031308, use (1.055 * value.powf(1.0 / 2.4)) - 0.055;
        For channel values lower than or equal to 0.0031308, use value * 12.92;
    Otherwise, the results should now be more consistent with other tools/engines.

The update_frame_count system should be placed in CorePlugin #
Rendering
Core
Time

The FrameCount resource was previously only updated when using the bevy_render feature. If you are not using this feature but still want the FrameCount it will now be updated correctly.
Pipelined Rendering #
Rendering
Tasks

App runner and SubApp extract functions are now required to be Send

This was changed to enable pipelined rendering. If this breaks your use case please report it as these new bounds might be able to be relaxed.
Remove ImageMode #
Rendering
UI

ImageMode never worked, if you were using it please create an issue.
Rename the background_color of 'ExtractedUiNodetocolor` #
Rendering
UI

The background_color field of ExtractedUiNode is now named color.
Remove the GlobalTransform::translation_mut method #
Transform
Hierarchy

GlobalTransform::translation_mut has been removed without alternative, if you were relying on this, update the Transform instead. If the given entity had children or parent, you may need to remove its parent to make its transform independent (in which case the new Commands::set_parent_in_place and Commands::remove_parent_in_place may be of interest)

Bevy may add in the future a way to toggle transform propagation on an entity basis.
Flip UI image #
UI

    UiImage is a struct now, so use UiImage::new(handler) instead of UiImage(handler)
    UiImage no longer implements Deref and DerefMut, so use &image.texture or &mut image.texture instead

Remove TextError::ExceedMaxTextAtlases(usize) variant #
UI

TextError::ExceedMaxTextAtlases(usize) was never thrown so if you were matching on this variant you can simply remove it.
Change default FocusPolicy to Pass #
UI

FocusPolicy default has changed from FocusPolicy::Block to FocusPolicy::Pass
Remove VerticalAlign from TextAlignment #
UI

The alignment field of Text now only affects the text’s internal alignment.

Change TextAlignment to TextAlignment` which is now an enum. Replace:

    TextAlignment::TOP_LEFT, TextAlignment::CENTER_LEFT, TextAlignment::BOTTOM_LEFT with TextAlignment::Left
    TextAlignment::TOP_CENTER, TextAlignment::CENTER_LEFT, TextAlignment::BOTTOM_CENTER with TextAlignment::Center
    TextAlignment::TOP_RIGHT, TextAlignment::CENTER_RIGHT, TextAlignment::BOTTOM_RIGHT with TextAlignment::Right

Changes for Text2dBundle

Text2dBundle has a new field text_anchor that takes an Anchor component that controls its position relative to its transform.

Text2dSize was removed. Use TextLayoutInfo instead.
Remove QueuedText #
UI

QueuedText was never meant to be user facing. If you relied on it, please make an issue.
Change the default width and height of Size to Val::Auto #
UI

The default values for Size width and height have been changed from Val::Undefined to Val::Auto. It’s unlikely to cause any issues with existing code.
Fix the Size helper functions using the wrong default value and improve the UI examples #
UI

The Size::width constructor function now sets the height to Val::Auto instead of Val::Undefined. The Size::height constructor function now sets the width to Val::Auto instead of Val::Undefined.
The size field of CalculatedSize should not be a Size #
UI

The size field of CalculatedSize has been changed to a Vec2.
Update winit to 0.28 #
Windowing

// 0.9
app.new()
    .add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            always_on_top: true,
            ..default()
        }),
        ..default()
    }));

// 0.10
app.new()
    .add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            window_level: bevy::window::WindowLevel::AlwaysOnTop,
            ..default()
        }),
        ..default()
    }));

