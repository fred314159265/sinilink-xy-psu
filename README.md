# Sinilink XY Power Supply Driver

## Introduction

This crate aims to be a somewhat complete library for interfacing with the XY series of programmable power supplies from Sinilink. It also aims to support `no_std` applications, making it suitable for controlling or monitoring the PSUs from bare metal devices such as microcontrollers.

This crate supports control over the UART/RS485 connection and not over Wi-Fi.

Some similar PSUs such as the DPS series (e.g. DPS5005) have a similar register setup, but with enough differences that this crate is incompatible (at the moment).

## ⚠⚠⚠ Current State ⚠⚠⚠

This repo is currently in early stages of development, is not tested and the API is very likely to change continually.

## Hardware

@TODO: document serial port pinout, voltage levels, etc.