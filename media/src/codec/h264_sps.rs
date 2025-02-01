use bitstream::bitstream::BitStream;

#[derive(Default, Debug)]
pub struct Sps {
    pub profile_idc: u8, //u(8)
    flag: u8,
    pub level_idc: u8,         // u(8)
    seq_parameter_set_id: u32, // ue(v)
    chroma_format_idc: u32, // ue(v)
    separate_colour_plane_flag: u8,           // u(1)
    bit_depth_luma_minus8: u32,               // ue(v)
    bit_depth_chroma_minus8: u32,             // ue(v)
    qpprime_y_zero_transform_bypass_flag: u8, // u(1)
    seq_scaling_matrix_present_flag: u8, // u(1)
    seq_scaling_list_present_flag: Vec<u8>, // u(1)
    log2_max_frame_num_minus4: u32, // ue(v)
    pic_order_cnt_type: u32,        // ue(v)
    log2_max_pic_order_cnt_lsb_minus4: u32, // ue(v)
    delta_pic_order_always_zero_flag: u8,       // u(1)
    offset_for_non_ref_pic: i32,                // se(v)
    offset_for_top_to_bottom_field: i32,        // se(v)
    num_ref_frames_in_pic_order_cnt_cycle: u32, // ue(v)
    offset_for_ref_frame: Vec<i32>, // se(v)
    max_num_ref_frames: u32,                  // ue(v)
    gaps_in_frame_num_value_allowed_flag: u8, // u(1)
    pic_width_in_mbs_minus1: u32,        // ue(v)
    pic_height_in_map_units_minus1: u32, // ue(v)
    frame_mbs_only_flag: u8,             // u(1)
    mb_adaptive_frame_field_flag: u8, // u(1)
    direct_8x8_inference_flag: u8, // u(1)
    frame_cropping_flag: u8, // u(1)
    frame_crop_left_offset: u32,   // ue(v)
    frame_crop_right_offset: u32,  // ue(v)
    frame_crop_top_offset: u32,    // ue(v)
    frame_crop_bottom_offset: u32, // ue(v)
    vui_parameters_present_flag: u8, // u(1)
}

impl From<&mut BitStream> for Sps {
    fn from(bs: &mut BitStream) -> Self{
        let mut sps = Sps::default();

        sps.profile_idc = bs.read_u(8) as u8;
        println!("profile_idc: {}", sps.profile_idc);
        sps.flag = bs.read_u(8) as u8;
        sps.level_idc = bs.read_u(8) as u8;
        println!("level_idc: {}", sps.level_idc);
        sps.seq_parameter_set_id = bs.read_ue();

        match sps.profile_idc {
            100 | 110 | 122 | 244 | 44 | 83 | 86 | 118 | 128 => {
                sps.chroma_format_idc = bs.read_ue();
                if sps.chroma_format_idc == 3 {
                    sps.separate_colour_plane_flag = bs.read_u1();
                }
                sps.bit_depth_luma_minus8 = bs.read_ue();
                sps.bit_depth_chroma_minus8 = bs.read_ue();

                sps.qpprime_y_zero_transform_bypass_flag = bs.read_u1();
                sps.seq_scaling_matrix_present_flag = bs.read_u1();

                if sps.seq_scaling_matrix_present_flag > 0 {
                    let matrix_dim: usize = if sps.chroma_format_idc != 2 {
                        8
                    } else {
                        12
                    };

                    for _ in 0..matrix_dim {
                        sps
                            .seq_scaling_list_present_flag
                            .push(bs.read_u1());
                    }
                }
            }
            _ => {}
        }

        sps.log2_max_frame_num_minus4 = bs.read_ue();
        sps.pic_order_cnt_type = bs.read_ue();

        match sps.pic_order_cnt_type {
            0 => {
                sps.log2_max_pic_order_cnt_lsb_minus4 =
                    bs.read_ue();
            }
            1 => {
                sps.delta_pic_order_always_zero_flag = bs.read_u1();
                sps.offset_for_non_ref_pic = bs.read_se();
                sps.offset_for_top_to_bottom_field = bs.read_se();
                sps.num_ref_frames_in_pic_order_cnt_cycle =
                    bs.read_ue();

                for i in 0..sps.num_ref_frames_in_pic_order_cnt_cycle as usize {
                    sps.offset_for_ref_frame[i] = bs.read_se();
                }
            }
            _ => {}
        }

        sps.max_num_ref_frames = bs.read_ue();
        sps.gaps_in_frame_num_value_allowed_flag = bs.read_u1();

        sps.pic_width_in_mbs_minus1 = bs.read_ue();
        sps.pic_height_in_map_units_minus1 = bs.read_ue();

        sps.frame_mbs_only_flag = bs.read_u1();

        if sps.frame_mbs_only_flag == 0 {
            sps.mb_adaptive_frame_field_flag = bs.read_u1();
        }
        sps.direct_8x8_inference_flag = bs.read_u1();
        sps.frame_cropping_flag = bs.read_u1();

        if sps.frame_cropping_flag > 0 {
            sps.frame_crop_left_offset = bs.read_ue();
            sps.frame_crop_right_offset = bs.read_ue();
            sps.frame_crop_top_offset = bs.read_ue();
            sps.frame_crop_bottom_offset = bs.read_ue();
        }

        sps.vui_parameters_present_flag = bs.read_u1();

        sps
    }
}

impl Sps {
    //! 
    pub fn parse_width_height(&self) -> (u32, u32) {
        let width = (self.pic_width_in_mbs_minus1 + 1) * 16
            - self.frame_crop_left_offset * 2
            - self.frame_crop_right_offset * 2;
        let height = ((2 - self.frame_mbs_only_flag as u32)
            * (self.pic_height_in_map_units_minus1 + 1)
            * 16)
            - (self.frame_crop_top_offset * 2)
            - (self.frame_crop_bottom_offset * 2);

        // log::trace!("parsed sps data: {:?}", self);
        (width, height)
    }  
}
