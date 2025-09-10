# Sinilink XY Power Supply Driver

## Introduction

This crate aims to be a somewhat complete library for interfacing with the XY series of programmable power supplies from Sinilink. It also aims to support `no_std` applications, making it suitable for controlling or monitoring the PSUs from bare metal devices such as microcontrollers.

This crate supports control over a UART/RS485 connection but not over Wi-Fi.

Some other PSUs such as the DPS series (e.g. DPS5005) have a similar register setup and also use Modbus RTU, but with enough differences that this crate is largely incompatible.

## Current State

This repo is currently in early stages of development, basic functionality has been tested but the API is very likely to change.

## Hardware

The power supplies use a 4-way Molex Picoblade connector, the TX line outputs 0-3.3V.


| Pin | Function | Notes                         |
|-----|----------|-------------------------------|
| 1   | VCC      | Outputs 5V.                   |
| 2   | TX       | Connect to RX on UART device. |
| 3   | RX       | Connect to TX on UART device. |
| 4   | GND      |                               |

<img src="assets\pinout.jpg" alt="Image showing Molex Picoblade and pinout with colours" style="zoom: 20%;">
