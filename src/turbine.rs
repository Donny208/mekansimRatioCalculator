use std::cmp::{max, min};
use log::debug;

const GENERAL_DISPERSER_GAS_FLOW: i32 = 1280; // mB/t
const GENERAL_VENT_GAS_FLOW: i32 = 32000; // mB/t
const GENERAL_CONDENSER_RATE: i32 = 64000; // mB/t

/// Struct Turbine
#[derive(Debug)]
pub struct Turbine {
    pub x_z: i32,
    pub y: i32,
    pub vents: i32,
    pub dispersers: i32,
    pub shaft_height: i32,
    pub blades: i32,
    pub coils: i32,
    pub capacity: i32,
    pub max_flow: i32,
    pub tank_volume: i32,
    pub max_production: f32,
    pub max_water_output: i32,
}

#[derive(Debug)]
struct TurbineFlow {
    shaft_height: i32,
    vents: i32,
    condensers: i32,
    max_flow: i32,
    max_water_ouput: i32,
}

///  Create turbine based on all blocks/parts added.  Mostly for calculating formulas
pub fn turbine_factory(
    x_z: i32,
    y: i32,
    condensers: i32,
    dispersers: i32,
    vents: i32,
    shaft_height: i32,
    blades: i32,
    coils: i32,
) -> Turbine {
    Turbine {
        x_z,
        y,
        vents,
        dispersers,
        shaft_height,
        blades,
        coils,
        capacity: energy_capacity(x_z, y),
        max_flow: calc_max_flow_rate(x_z, shaft_height, vents),
        tank_volume: calc_lower_volume(x_z, shaft_height),
        max_production: 0.0,
        max_water_output: max_water_output(condensers),
    }
}

//FLOW = min(1, TURBINE_STORED_AMOUNT / MAX_RATE) *
//          (TURBINE_STORED_AMOUNT/TURBINE_MAX_STORED_AMOUNT) * MAX_RATE

///  Return most optimal turbine only based on user inputing dimensions
pub fn optimal_turbine_with_dimensions(x_z: i32, y: i32) -> Turbine {
    // Check if turbine's dimensions fall within an acceptable size
    // TODO Need to throw an error, return nothing
    if x_z < 5 {
        println!("Reactor length and width too small, min 5 by 5 blocks.");
    } else if 17 < x_z {
        println!("Reactor length and width too large, min 5 by 5 blocks.");
    }

    if y < 5 {
        println!("Reactor height too small, min 5 blocks.");
    } else if 17 < y {
        println!("Reactor height too large, max 18 blocks.");
    }

    // Calculate the max flow, and max water output for each shaft_height of the turbine.
    let info = (1..min(2 * y - 5, 14)).map( | shaft_height: i32 | {
        let tank_flow = calc_tank_flow_rate(x_z, shaft_height);
        let mut smallest_difference = i32::MAX;
        let mut best_vent_count = 0;
        // Find the vent_count/vent flow closest to the tank flow.
        for vent_count in (0..calc_max_vents(x_z, y, shaft_height)).rev() {
            let vent_flow = calc_vent_flow_rate(vent_count);
            // Get absolute
            debug!("Tank Flow: {tank_flow}, Vent_flow: {vent_flow}");
            if tank_flow - vent_flow < smallest_difference {
                smallest_difference = (tank_flow - vent_flow).abs();
                best_vent_count = vent_count;
            }
            // TODO Can we break?
        }
        //TODO Now figure out what the best number of condensors would be
        let max_flow = calc_max_flow_rate(x_z, shaft_height, best_vent_count);
        let condensers = calc_optimal_condensers(x_z, y, shaft_height, shaft_height * 2, max_flow);
        let water_output = max_water_output(condensers);
        TurbineFlow {shaft_height, vents: best_vent_count, condensers, max_flow, max_water_ouput: water_output}
    }).filter(| x | x.condensers > 0).collect::<Vec<TurbineFlow>>();
    for i in &info{
        println!("TurbineFlow Info: {:?}", i);
    }
    let best_turbine = info.iter().max_by_key(| x | x.max_flow).unwrap();
    let shaft_height = best_turbine.shaft_height;
    let max_rate = best_turbine.max_flow;
    let vents = best_turbine.vents;
    
    // TODO Man I hate this calculation, it should be done better.
    // Find the ideal ratio of vents vs volume
    // Maximum shaft height = min(2xLENGTH-5,14) [so blades don't touch sides]
    // maxrate, vent_count, shaft_height

    let coils = calc_coils_needed(shaft_height * 2);
    let condensers = calc_optimal_condensers(x_z, y, shaft_height, coils, max_rate);
    Turbine {
        x_z,
        y,
        vents,
        dispersers: calc_pressure_dispersers(x_z),
        shaft_height: shaft_height,
        blades: shaft_height * 2,
        coils,
        capacity: 0,
        max_flow: max_rate,
        tank_volume: calc_lower_volume(x_z, shaft_height),
        max_production: 0.0,
        max_water_output: max_water_output(condensers),
    }
}

// Max Flow Rate
//MAX_RATE = min(TURBINE_DISPERSER_COUNT * GENERAL_DISPERSER_GAS_FLOW * structure.lowerVolume,
//               TURBINE_VENT_COUNT * GENERAL_VENT_GAS_FLOW)
fn calc_max_flow_rate(x_z: i32, shaft_height: i32, vent_count: i32) -> i32 {
    let tank_flow: i32 = calc_pressure_dispersers(x_z)
        * GENERAL_DISPERSER_GAS_FLOW
        * calc_lower_volume(x_z, shaft_height);
    let vent_flow: i32 = vent_count * GENERAL_VENT_GAS_FLOW;
    debug!("Tank flow: {tank_flow}");
    debug!("Vent flow: {vent_flow}");
    min(tank_flow, vent_flow)
}

/// Calculate the lower tank's flow rate
fn calc_tank_flow_rate(x_z: i32, shaft_height: i32) -> i32 {
    calc_pressure_dispersers(x_z)
        * GENERAL_DISPERSER_GAS_FLOW
        * calc_lower_volume(x_z, shaft_height)
}

/// Calculate the flow rate of the vents
fn calc_vent_flow_rate(vent_count: i32) -> i32 {
    vent_count * GENERAL_VENT_GAS_FLOW
}

/// Calculate the max number of vents
fn calc_max_vents(x_z: i32, y: i32, shaft_height: i32) -> i32 {
    let top_vents = (x_z - 2).pow(2);
    let remaining_height = y - 2 - shaft_height;
    let side_vents = (remaining_height * (x_z - 2)) * 4;
    top_vents + side_vents
}
fn calc_lower_volume(x_z: i32, shaft_height: i32) -> i32 {
    x_z * x_z * shaft_height
}

///
fn calc_coils_needed(num_blades: i32) -> i32 {
    max((num_blades as f32 / 4.0).ceil() as i32, 2)
}

///
fn calc_pressure_dispersers(x_z: i32) -> i32 {
    (x_z - 2).pow(2) - 1
}

///
fn energy_capacity(x_z: i32, y: i32) -> i32 {
    x_z.pow(2) * y * 16000
}

fn max_energy_production() {}

// fn calc_flow_rate(num_vents: i32) -> i32 {
//     // Flow rate is 16,000 MB/t
//     num_vents * 16000
// }

//fn calc_max_water_output() -> i32 {
//    // Don't know yet.
//}

fn calc_optimal_condensers(x_z: i32, y: i32, shaft_height: i32, coils: i32, max_flow: i32) -> i32 {
    debug!("y: {y}, shaft_height: {shaft_height}");
    let remaining_y = (y - 3) - shaft_height;
    let avaliable_space = remaining_y * (x_z - 2).pow(2) - coils;
    debug!("{remaining_y}");
    debug!("{avaliable_space}");
    min(max_flow / GENERAL_CONDENSER_RATE, avaliable_space)
}

fn max_water_output(condensers: i32) -> i32 {
    debug!("Condensers: {condensers}");
    condensers * GENERAL_CONDENSER_RATE
}

#[cfg(test)]
mod tests {
    use super::*;

    //Arrange
    //Act
    //Assert

    #[test]
    fn test_calc_coils_needed() {
        let blades = 10;
        let expected_coils = 3;
        assert_eq!(calc_coils_needed(blades), expected_coils);
    }

    #[test]
    fn test_calc_pressure_dispersers() {
        let x_z = 5;
        let expected = 15;
        assert_eq!(calc_pressure_dispersers(x_z), expected);
    }

    #[test]
    fn test_calc_max_vents() {
        // A 5x5x5 with shaft of 1 tall could have a max vent of 33
        let x_z = 5;
        let y = 5;
        let shaft_height = 1;
        let expected = 33;
        let actual = calc_max_vents(x_z, y, shaft_height);
        assert_eq!(actual, expected);
        // A 9x9x11 with shaft 5 tall could have max vents of 161
        let x_z = 9;
        let y = 11;
        let shaft_height = 5;
        let expected = 161;
        let actual = calc_max_vents(x_z, y, shaft_height);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_turbine_factory() {
        let actual = turbine_factory(9, 11, 48, 48, 105, 5, 10, 2);
        // assert_eq!(actual.capacity, 25920000);
        assert_eq!(actual.max_flow, 3360000);
        assert_eq!(actual.tank_volume, 405);
        assert_eq!(actual.dispersers, 48);
        assert_eq!(actual.vents, 105);
        assert_eq!(actual.coils, 2);
        // assert_eq!(actual.max_production, 3.83);
        assert_eq!(actual.max_water_output, 3072000);
    }
    #[test]
    fn test_optimal_turbine_with_dimensions() {
        let x_z = 5;
        let y = 5;
        let expected = Turbine {
            x_z,
            y,
            vents: 8,
            dispersers: 8,
            shaft_height: 1,
            blades: 2,
            coils: 2,
            capacity: 1600000,
            max_flow: 256000,
            tank_volume: 25,
            max_production: 73.13, //kJ
            max_water_output: 256000,
        };
        let actual = optimal_turbine_with_dimensions(x_z, y);
            // assert_eq!(actual.capacity, 25920000);
            assert_eq!(actual.max_flow, 256000);
            assert_eq!(actual.tank_volume, 25);
            assert_eq!(actual.dispersers, 8);
            assert_eq!(actual.vents, 8);
            assert_eq!(actual.coils, 2);
            // assert_eq!(actual.max_production, 3.83);
            assert_eq!(actual.max_water_output, 256000);
    }
}
