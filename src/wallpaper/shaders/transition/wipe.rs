crate::simple_transition!(
    Wipe,

    [
        t_diffuse_old, s_diffuse_old,
        t_diffuse_new, s_diffuse_new
    ],

    data { angle: f32 => angle: f32 },

    |in { window_position, ndc_position, texture_coords }| => {"
        let angle = radians(data.angle);
        let progress = data.progress;
        let position = vec2(in.ndc_position[0], in.ndc_position[1]);

        let d = vec2(cos(angle), sin(angle));
        let proj = dot(position, d);
        let threshold = mix(-1.414, 1.414, progress);

        if proj > threshold {
            return textureSample(t_diffuse_old, s_diffuse_old, in.texture_coords);
        } else {
            return textureSample(t_diffuse_new, s_diffuse_new, in.texture_coords);
        }
    "}
);
