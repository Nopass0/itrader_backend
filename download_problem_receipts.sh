#!/bin/bash

# Download problematic receipts for analysis

mkdir -p test_data/problem_receipts

# T-Bank receipts that failed to extract bank
echo "Downloading T-Bank receipts..."
wget -O test_data/problem_receipts/2462986_tbank.pdf https://cdn.gate.cx/3528737/Receipt.pdf
wget -O test_data/problem_receipts/2462967_tbank.pdf https://cdn.gate.cx/3528425/Receipt.pdf  
wget -O test_data/problem_receipts/2462923_tbank.pdf https://cdn.gate.cx/3528717/Receipt.pdf

# Promsvyazbank receipts
echo "Downloading Promsvyazbank receipts..."
wget -O test_data/problem_receipts/2462596_psb.pdf https://cdn.gate.cx/3528435/Receipt.pdf
wget -O test_data/problem_receipts/2452921_psb.pdf https://cdn.gate.cx/3511717/Receipt.pdf

# MTS Bank receipt
echo "Downloading MTS Bank receipt..."
wget -O test_data/problem_receipts/2463546_mts.pdf https://cdn.gate.cx/3530265/Receipt.pdf

# VTB receipt
echo "Downloading VTB receipt..."
wget -O test_data/problem_receipts/2449415_vtb.pdf https://cdn.gate.cx/3504991/Receipt-(3345).pdf

echo "Done downloading problem receipts"