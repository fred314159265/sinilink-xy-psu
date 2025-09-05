# Sinilink XY Power Supply Driver

## Introduction

This crate aims to be a somewhat complete library for interfacing with the XY series of programmable power supplies from Sinilink. It also aims to support `no_std` applications, making it suitable for controlling or monitoring the PSUs from bare metal devices such as microcontrollers.

This crate supports control over a UART/RS485 connection but not over Wi-Fi.

Some other PSUs such as the DPS series (e.g. DPS5005) have a similar register setup and also use Modbus RTU, but with enough differences that this crate is largely incompatible.

## Current State

This repo is currently in early stages of development, basic functionality has been tested but the API is very likely to change continually.

## Hardware

@TODO: document serial port pinout, voltage levels, etc.