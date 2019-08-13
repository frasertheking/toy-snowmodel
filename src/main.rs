
extern crate gnuplot;
extern crate csv;

use std::error::Error;
use csv::Writer;
use gnuplot::*;
// use std::fmt;

// struct Array<T> {
//     data: [T; 50]
// }

// impl<T: fmt::Debug> fmt::Debug for Array<T> {
//     fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//         self.data[..].fmt(formatter)
//     }
// }

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


fn main() {

    let mut outputs: [f64; 50] = [-1.0; 50];
    let mut outputs2: [f64; 50] = [-1.0; 50];
    let mut outputs3: [f64; 50] = [-1.0; 50];
    let mut temps: [f64; 50] = [-1.0; 50];
    for x in (0..50).rev() {
    	temps[x] = x as f64;
    	outputs[x] = run_model(x as f64, false).total_water_output;
    	outputs2[x] = run_model(x as f64, false).total_ablation;
    	outputs3[x] = run_model(x as f64, false).total_melt;
	}

	example(outputs, outputs2, outputs3);

	// let array = Array { data: outputs };
	// println!("{:?}", array);

 //    let mut fg = Figure::new();
	// fg.axes2d()
	// 	.set_title("A plot", &[])
	// 	.set_legend(Graph(0.5), Graph(0.9), &[], &[])
	// 	.set_x_label("x", &[])
	// 	.set_y_label("y^2", &[])
	// 	.lines(
	// 		&temps[0 .. 50],
	// 		&outputs[0 .. 50],
	// 		&[Caption("12341234234")],
	// 	);
	// fg.axes2d()
	// 	.set_legend(Graph(0.5), Graph(0.9), &[], &[])
	// 	.lines(
	// 		&temps[0 .. 50],
	// 		&outputs2[0 .. 50],
	// 		&[Caption("asdasdasdas")],
	// 	);
	// fg.axes2d()
	// 	.set_legend(Graph(0.5), Graph(0.9), &[], &[])
	// 	.lines(
	// 		&temps[0 .. 50],
	// 		&outputs3[0 .. 50],
	// 		&[Caption("gfdgdsfgsdf")],
	// 	);
	// fg.show();
}

fn example(arr1: [f64; 50], arr2: [f64; 50], arr3: [f64; 50]) -> Result<(), Box<Error>> {
    let mut wtr = Writer::from_path("foo.csv")?;
    wtr.serialize(&arr1[0 .. 50])?;
    wtr.serialize(&arr2[0 .. 50])?;
    wtr.serialize(&arr3[0 .. 50])?;
    wtr.flush()?;
    Ok(())
}

fn run_model(air_temperature: f64, model_diog: bool) -> ModelRun {
	// Print text to the console
	if model_diog {
    	println!("---Simple Snow Melt Model---");
    }

	// Site variables
	let measurement_height: f64 = 2.0;       // meters
	let roughness_height: f64 = 0.0015;      // meters
	let forest_cover_fraction: f64 = 0.8;
	let albedo: f64 = 0.55;
	let snow_density: f64 = 500.0;           // kg/m^3

	// Weather variables
	let clear_sky_solar_rad: f64 = 14.3;     // MJ/(m^2 day)
	let cloud_cover_fraction: f64 = 0.5;
	let relative_humidity: f64 = 0.8;        // Wa
	let wind_speed: f64 = 6.0;               // m/s
	let rain_rate: f64 = 10.0;               // mm/day
	let atmospheric_pressure: f64 = 101.3;   // kPa

    let net_solar_rad: f64 = calc_net_solar_rad(clear_sky_solar_rad, cloud_cover_fraction, forest_cover_fraction, albedo);
    let vapor_pressure: f64 = calc_vap_pressure(air_temperature, relative_humidity);
    let atmos_emissivity: f64 = calc_atmos_emissivity(forest_cover_fraction, vapor_pressure, air_temperature, cloud_cover_fraction);
    let net_long_wave_rad: f64 = calc_net_long_rad(atmos_emissivity, air_temperature);
    let net_rad: f64 = net_solar_rad + net_long_wave_rad;
    let adjusted_wind_speed: f64 = calc_adj_wind_speed(wind_speed, forest_cover_fraction);
    let air_density: f64 = calc_air_density(atmospheric_pressure, air_temperature);
    let richardson_num: f64 = calc_richardson_num(measurement_height, roughness_height, air_temperature, adjusted_wind_speed);
    let stability_factor_m: f64 = calc_stability_factor_m(richardson_num);
    let stability_factors_v_h: f64 = calc_stability_factors_v_h(richardson_num, stability_factor_m);
    let sensible_heat_xfer_co: f64 = calc_sensible_heat_xfer_co(air_density, measurement_height, roughness_height);
    let latent_heat_xfer_co: f64 = calc_latent_heat_xfer_co(air_density, atmospheric_pressure, measurement_height, roughness_height);
    let sensible_xfer_rate: f64 = calc_sensible_heat_xfer_rate(sensible_heat_xfer_co, adjusted_wind_speed, air_temperature, stability_factor_m, stability_factors_v_h);
    let latent_xfer_rate: f64 = calc_latent_heat_xfer_rate(latent_heat_xfer_co, adjusted_wind_speed, vapor_pressure, stability_factor_m, stability_factors_v_h);
    let condensation: f64 = calc_condensation(latent_xfer_rate);
    let rain_heat: f64 = calc_rain_heat(rain_rate, air_temperature);
    let total_heat_input_rate: f64 = calc_total_heat_input_rate(net_rad, sensible_xfer_rate, latent_xfer_rate, rain_heat);

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
	    println!("\nSite:\nMeasurement Height: {}\nRoughness Height: {}\nForest Cover: {}\nAlbedo: {}\nSnowpack Density: {}", measurement_height, roughness_height, forest_cover_fraction, albedo, snow_density);
	    println!("\nWeather:\nClear Sky Solar Rad: {}\nCloud Fraction: {}\nAir Temp: {}\nRelative Humidity: {}\nWind Speed: {}\nRain Rate: {}\nAtmos Pressure: {}", clear_sky_solar_rad, cloud_cover_fraction, air_temperature, relative_humidity, wind_speed, rain_rate, atmospheric_pressure);

	    // OUTPUT VARIABLES
	    println!("\n\nOUTPUT:");
	    println!("\nEnergy Balance Approach: \nTotal Melt: {} \nTotal Ablation: {} \nTotal Water Output: {}", total_melt, total_ablation, total_water_output);
	    println!("\nTemperature Index Approach: \nTotal Melt: {} \nTotal Water Output: {}", ti_total_melt, ti_total_water_output);
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
	return kcs*(0.355 + 0.68*(1.0-c))*(-3.91*f).exp()*(1.0-a);
}

fn calc_vap_pressure(ta: f64, wa: f64) -> f64 {
	return 0.611*(17.3*ta/(ta+237.3)).exp()*wa;
}

fn calc_atmos_emissivity(f: f64, ea: f64, ta: f64, c: f64) -> f64 {
	return (1.0-f)*1.72*(f64::powf(ea/(ta+273.2), 1.0/7.0))*(1.0+0.22*(f64::powf(c, 2.0))) + f;
}

fn calc_net_long_rad(est: f64, ta: f64) -> f64 {
	return est*0.0000000049*f64::powf(ta+273.2, 4.0) - 27.3;
}

fn calc_adj_wind_speed(vao: f64, f: f64) -> f64 {
	return vao*(1.0-0.8*f);
}

fn calc_air_density(p: f64, ta: f64) -> f64 {
	return p/(0.288*(ta+273.2));
}

fn calc_richardson_num(za: f64, zo: f64, ta: f64, va: f64) -> f64 {
	return (2.0*9.81*(za-zo)*ta)/((ta+2.0*273.2)*f64::powf(va, 2.0));
}

fn calc_stability_factor_m(rj: f64) -> f64 {
	if rj > 0.0 {
		return 1.0 / (1.0-5.2*rj);
	} else {
		return 1.0 / f64::powf(1.0-18.0*rj, 0.25);
	}
}

fn calc_stability_factors_v_h(rj: f64, m: f64) -> f64 {
	if rj < -0.03 {
		return 1.3*m;
	} else {
		return m;
	}
}

fn calc_sensible_heat_xfer_co(pa: f64, za: f64, zo: f64) -> f64 {
	return 0.001005*pa*0.16/(f64::powf((za/zo).ln(), 2.0))*86400.0;
}

fn calc_latent_heat_xfer_co(pa: f64, p: f64, za: f64, zo: f64) -> f64 {
	return 2.47*0.622*pa/p*0.16/f64::powf((za/zo).ln(), 2.0)*86400.0;
}

fn calc_sensible_heat_xfer_rate(kh: f64, va: f64, ta: f64, m: f64, vh: f64) -> f64 {
	return kh*va*ta/(m*vh);
}

fn calc_latent_heat_xfer_rate(kle: f64, va: f64, ea: f64, m: f64, vh: f64) -> f64 {
	return kle*va*(ea-0.611)/(m*vh);
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








