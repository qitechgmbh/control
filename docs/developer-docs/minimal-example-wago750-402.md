# Minimal Example — Digital Input WAGO750-402

## Table of Contents
1. [Introduction](#1-introduction)
2. [Requirements](#2-requirements)
3. [Hardware Setup](#3-hardware-setup)
4. [Software Setup](#4-software-setup)
5. [Demo](#5-demo)
6. [References](#6-references)

## 1. Introduction

## 2. Requirements

### Hardware
- WAGO 750-354,
- WAGO 750-602,
- WAGO 750-402,
- WAGO 750-600
- WAGO 750-1506

### Software
*(Installation steps in Section 4)*

- Rust tollchain
- Node-js + npm
- QiTech Control repository
- EtherCAT HAL *(include inside repo)*

## 3. Hardware Setup

### 3.1 Schematic

#### ⚠️ Safety Warning  
Always disconnect power before wiring.  
Working on live EtherCAT terminals can cause serious damage or electrical shock.

### 3.2.1 Safe Wiring Procedure (Beckhoff Recommended)
1. Insert a screwdriver **straight** into the square release hole.  
2. Insert the stripped wire into the round opening.  
3. Remove the screwdriver — the spring clamp locks the wire.

![](../assets/wiring.png)

---

### 3.2.2 Wiring Steps (Used in This Example)
 
#### **Figure 1 — WAGO Minimal Wiring**

### 3.3 WAGO Integration

#### **Figure 2 — WAGO Terminal**

### 3.4 Final Assembled Setup

#### **Figure 3 — Connected**

## 3.5 Power & Ethernet

### 3.5.1 Power  

#### Example AC/DC Adapter (Figure 4):

### 3.5.2 Ethernet  

## 4. Software Setup

### 4.1 Installing on Ubuntu/Debian

### 4.2 Running the Backend

### 4.3 Running the Frontend

## 5. Demo

### 5.1 Assigning Devices in the Dashboard

## 6. References