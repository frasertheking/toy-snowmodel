
///////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////
//////                                                        //////
//////       Script to run simple snow model with both        //////
//////       a physical-based approach and temperature        //////
//////       index approach.                                  //////
//////                                                        //////
//////       This is a rust script: rust is freely            //////
//////       available from https://www.rust-lang.org.        //////
//////       To run the script, cd into the debug             //////
//////       directory of this project and type               //////
//////                                                        //////
//////       $ cargo build                                    //////
//////       $ ./model                                        //////
//////                                                        //////
//////       at the prompt. Your model output will be         //////
//////	     saved in a .csv file in the same directory       //////
//////                                                        //////
//////                               Fraser King              //////
//////                                 August 15, 2019        //////
//////                                                        //////
////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////
//
// General formulas retrieved from Dingman, 2015
//

extern crate gnuplot;
extern crate csv;
use std::error::Error;
use csv::Writer;

// Model Properties
struct ModelRun {
	net_solar_rad: f64,
	vapor_pressure: f64,
	atmos_emissivity: f64,
	net_long_wave_rad: f64,
	net_rad: f64,
	adjusted_wind_speed: f64,
	air_density: f64,
	richardson_num: f64,
	stability_factor_m: f64,
	stability_factors_v_h: f64,
	sensible_heat_xfer_co: f64,
	latent_heat_xfer_co: f64,
	sensible_xfer_rate: f64,
	latent_xfer_rate: f64,
	condensation: f64,
	rain_heat: f64,
	total_heat_input_rate: f64,
    total_melt: f64,
    total_ablation: f64,
    total_water_output: f64,
    ti_total_melt: f64,
    ti_total_water_output: f64,
}

// Saving helper function
fn save_to_disk(temperatures: [f64; 50],
			  total_water_arr: [f64; 50],
			  total_abl_arr: [f64; 50],
			  total_melt_arr: [f64; 50],
			  ti_total_abl_arr: [f64; 50],
			  ti_total_melt_arr: [f64; 50]) -> Result<(), Box<Error>> {
    let mut wtr = Writer::from_path("model_output.csv")?;
    wtr.serialize(&temperatures[0 .. 50])?;
    wtr.serialize(&total_water_arr[0 .. 50])?;
    wtr.serialize(&total_abl_arr[0 .. 50])?;
    wtr.serialize(&total_melt_arr[0 .. 50])?;
    wtr.serialize(&ti_total_abl_arr[0 .. 50])?;
    wtr.serialize(&ti_total_melt_arr[0 .. 50])?;
    wtr.flush()?;
    Ok(())
}

// Run model
fn run_model(air_temperature: f64, model_diog: bool) -> ModelRun {
	// Print text to the console
	if model_diog {
    	println!("---Simple Snow Melt Model---");
    }

    ///////////////////////////////////////
    //// Variable initialization
    //// Feel free to play with these values

	// Site variables
	let measurement_height: f64 = 5.0;       // meters
	let roughness_height: f64 = 0.0015;      // meters
	let forest_cover_fraction: f64 = 0.8;
	let albedo: f64 = 0.55;
	let snow_density: f64 = 500.0;           // kg/m^3

	// Weather variables
	let clear_sky_solar_rad: f64 = 14.3;     // MJ/(m^2 day)
	let cloud_cover_fraction: f64 = 0.5;
	let relative_humidity: f64 = 0.8;        // Wa
	let wind_speed: f64 = 6.0;               // m/s
	let rain_rate: f64 = 0.0;               // mm/day
	let atmospheric_pressure: f64 = 101.3;   // kPa

    let net_solar_rad: f64 = calc_net_solar_rad(clear_sky_solar_rad, 
    	cloud_cover_fraction, forest_cover_fraction, albedo);

    let vapor_pressure: f64 = calc_vap_pressure(air_temperature, relative_humidity);

    let atmos_emissivity: f64 = calc_atmos_emissivity(forest_cover_fraction, 
    	vapor_pressure, 
    	cloud_cover_fraction);

    let net_long_wave_rad: f64 = calc_net_long_rad(atmos_emissivity, air_temperature);

    let net_rad: f64 = net_solar_rad + net_long_wave_rad;

    let adjusted_wind_speed: f64 = calc_adj_wind_speed(wind_speed, forest_cover_fraction);

    let air_density: f64 = calc_air_density(atmospheric_pressure, air_temperature);

    let richardson_num: f64 = calc_richardson_num(air_temperature, adjusted_wind_speed);

    let stability_factor_m: f64 = calc_stability_factor_m(measurement_height, roughness_height);

    let stability_factors_v_h: f64 = calc_stability_factors_v_h(richardson_num, stability_factor_m);

    let sensible_heat_xfer_co: f64 = calc_sensible_heat_xfer_co(air_density, 
    	measurement_height, roughness_height);

    let latent_heat_xfer_co: f64 = calc_latent_heat_xfer_co(air_density, 
    	atmospheric_pressure, measurement_height, roughness_height);

    let sensible_xfer_rate: f64 = calc_sensible_heat_xfer_rate(stability_factors_v_h,
    	sensible_heat_xfer_co, adjusted_wind_speed, air_temperature);

    let latent_xfer_rate: f64 = calc_latent_heat_xfer_rate(stability_factors_v_h, 
    	latent_heat_xfer_co, adjusted_wind_speed, vapor_pressure);

    let condensation: f64 = calc_condensation(latent_xfer_rate);

    let rain_heat: f64 = calc_rain_heat(rain_rate, air_temperature);

    let total_heat_input_rate: f64 = calc_total_heat_input_rate(net_rad, 
    	sensible_xfer_rate, latent_xfer_rate, rain_heat);

    // Energy Balance Approach
    let total_melt: f64 = calc_total_melt(total_heat_input_rate);
    let total_ablation: f64 = calc_total_ablation(latent_xfer_rate, total_melt, condensation);
    let total_water_output: f64 = calc_water_output(latent_xfer_rate, total_melt, rain_rate, condensation);

    // Temperature Index Approach
    let ti_total_melt: f64 = calc_ti_total_melt(air_temperature, forest_cover_fraction, snow_density);
    let ti_total_water_output: f64 = calc_ti_total_water_output(ti_total_melt, rain_rate);

    if model_diog {
	    // INPUT VARIABLES
	    println!("\n\nINPUT:");
	    println!("\nSite:\nMeasurement Height: {}\nRoughness Height: 
	    	     {}\nForest Cover: {}\nAlbedo: {}\nSnowpack Density: {}",
	     measurement_height, roughness_height, forest_cover_fraction, albedo, snow_density);
	    println!("\nWeather:\nClear Sky Solar Rad: {}\nCloud Fraction: {}\nAir Temp: 
	    	{}\nRelative Humidity: {}\nWind Speed: {}\nRain Rate: {}\nAtmos Pressure: {}",
	     clear_sky_solar_rad, cloud_cover_fraction, air_temperature, relative_humidity,
	     wind_speed, rain_rate, atmospheric_pressure);

	    // OUTPUT VARIABLES
	    println!("\n\nOUTPUT:");
	    println!("\nEnergy Balance Approach: \nTotal Melt: {} \nTotal Ablation: 
	    	{} \nTotal Water Output: {}", total_melt, total_ablation, total_water_output);
	    println!("\nTemperature Index Approach: \nTotal Melt: {} \nTotal Water Output: {}",
	     ti_total_melt, ti_total_water_output);
	    println!("\n\n---Complete---");
	}

	let model_run: ModelRun = ModelRun {
		net_solar_rad: net_solar_rad,
		vapor_pressure: vapor_pressure,
		atmos_emissivity: atmos_emissivity,
		net_long_wave_rad: net_long_wave_rad,
		net_rad: net_rad,
		adjusted_wind_speed: adjusted_wind_speed,
		air_density: air_density,
		richardson_num: richardson_num,
		stability_factor_m: stability_factor_m,
		stability_factors_v_h: stability_factors_v_h,
		sensible_heat_xfer_co: sensible_heat_xfer_co,
		latent_heat_xfer_co: latent_heat_xfer_co,
		sensible_xfer_rate: sensible_xfer_rate,
		latent_xfer_rate: latent_xfer_rate,
		condensation: condensation,
		rain_heat: rain_heat,
		total_heat_input_rate: total_heat_input_rate,
		total_melt: total_melt,
		total_ablation: total_ablation,
		total_water_output: total_water_output,
		ti_total_melt: ti_total_melt,
		ti_total_water_output: ti_total_water_output
	};

    return model_run;
}



//////////////////////////////////////////////
// ENERGY BALANCE FNS:

fn calc_total_melt(s: f64) -> f64 {
	if s < 0.0 {
		return 0.0;
	} else {
		return s / 0.334;
	}
}

fn calc_total_ablation(le: f64, s: f64, condensation: f64) -> f64 {
	if le < 0.0 {
		return s + condensation;
	} else {
		return s;
	}
}

fn calc_water_output(le: f64, total_melt: f64, r: f64, condensation: f64) -> f64 {
	if le < 0.0 {
		return total_melt + r + condensation
	} else {
		return total_melt + r
	}
}



//////////////////////////////////////////////
// TEMPERATURE INDEX FNS:

fn calc_ti_total_melt(ta: f64, f: f64, ps: f64) -> f64 {
	if ta > 0.0 {
		return f*(19.6*ps/1000.0-2.39)*ta+(1.0-f)*(10.4*ps/1000.0-0.7)*ta;
	} else {
		return 0.0;
	}
}

fn calc_ti_total_water_output(ti_total_melt: f64, r: f64) -> f64 {
	return ti_total_melt + r;
}



//////////////////////////////////////////////
// ENERGY BALANCE HELPER FNS:

fn calc_net_solar_rad(kcs: f64, c: f64, f: f64, a: f64) -> f64 {
	// Energy-Exchange Processes

	// Shortwave (Solar Radiation)
	// K = K_incident - K_refl = K_incident * (1 - albedo) 

	// Cloud cover (empircal values from Croley 1989)
	// Tau_c = 0.355 + 0.68 * (1 - c)

	// Forest cover insulation (value for lodge-pole pine Mahat and Tarborton 2012)
	// Tau_f = exp(-3.91 * F)
	return kcs*(0.355 + 0.68*(1.0-c))*(-3.91*f).exp()*(1.0-a);
}

fn calc_vap_pressure(ta: f64, wa: f64) -> f64 {
	// Saturation vapor pressure
	return 0.611*(17.3*ta/(ta+237.3)).exp()*wa;
}

fn calc_atmos_emissivity(f: f64, ea: f64, c: f64) -> f64 {
	return (1.0-f)*((1.0-0.84*c)*(0.83-0.18*(-1.54*ea).exp())+0.84*c)+f
}

fn calc_net_long_rad(est: f64, ta: f64) -> f64 {
	return est*0.0000000049*f64::powf(ta+273.2, 4.0) - 0.0000000049*f64::powf(273.2, 4.0);
}

fn calc_adj_wind_speed(vao: f64, f: f64) -> f64 {
	return vao*(1.0-0.8*f);
}

fn calc_air_density(p: f64, ta: f64) -> f64 {
	return p/(0.288*(ta+273.2));
}

fn calc_richardson_num(ta: f64, va: f64) -> f64 {
	// Stability state of the atmosphere 

	return (f64::powf(2.0, 2.0)*9.81*ta)/(0.5*(ta+2.0*273.2)*f64::powf(va, 2.0));
}

fn calc_stability_factor_m(za: f64, zo: f64) -> f64 {
	return 1.0 / ((za/zo).ln()+5.0);
}

fn calc_stability_factors_v_h(rj: f64, m: f64) -> f64 {
	if rj < m {
		return f64::powf(1.0-rj/0.2, 2.0);
	} else {
		return f64::powf(1.0-m/0.2, 2.0);
	}
}

fn calc_sensible_heat_xfer_co(pa: f64, za: f64, zo: f64) -> f64 {
	return (0.622*f64::powf(0.4, 2.0)*pa*0.001005/(f64::powf((za/zo).ln(), 2.0)))*86400.0;
}

fn calc_latent_heat_xfer_co(pa: f64, p: f64, za: f64, zo: f64) -> f64 {
	return (0.622*pa*2.47*f64::powf(0.4, 2.0)/(p*f64::powf((za/zo).ln(), 2.0)))*86400.0;
}

fn calc_sensible_heat_xfer_rate(vh: f64, kh: f64, uaf: f64, ta: f64) -> f64 {
	return vh*kh*uaf*ta;
}

fn calc_latent_heat_xfer_rate(vh: f64, kle: f64, uaf: f64, ea: f64) -> f64 {
	return vh*kle*uaf*(ea-0.611);
}

fn calc_condensation(le: f64) -> f64 {
	return le.abs()/2.47;
}

fn calc_rain_heat(r: f64, ta: f64) -> f64 {
	return r*ta*0.004187;
}

fn calc_total_heat_input_rate(net_rad: f64, h: f64, le: f64, r: f64) -> f64 {
	return net_rad + h + le + r;
}



//////////////////////////////////////////////
// MAIN RUNLOOP:
fn main() {

	// Model Run
	// This example is running over 50 temperatures 0 -> 50 degrees C
	let mut total_water_arr: [f64; 50] = [-1.0; 50];
	let mut total_abl_arr: [f64; 50] = [-1.0; 50];
	let mut total_melt_arr: [f64; 50] = [-1.0; 50];
	let mut ti_total_abl_arr: [f64; 50] = [-1.0; 50];
	let mut ti_total_melt_arr: [f64; 50] = [-1.0; 50];
	let mut temperatures: [f64; 50] = [-1.0; 50];
	for x in (0..50).rev() {
		temperatures[x] = x as f64;
		total_water_arr[x] = run_model(x as f64, false).total_water_output;
		total_abl_arr[x] = run_model(x as f64, false).total_ablation;
		total_melt_arr[x] = run_model(x as f64, false).total_melt;
		ti_total_abl_arr[x] = run_model(x as f64, false).ti_total_melt;
		ti_total_melt_arr[x] = run_model(x as f64, false).ti_total_water_output;
	}

	save_to_disk(temperatures, total_water_arr, total_abl_arr, total_melt_arr, ti_total_abl_arr, ti_total_melt_arr);
}








