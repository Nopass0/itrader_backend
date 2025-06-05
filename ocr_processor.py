"""
OCR Processor for Receipt Images
Handles OCR processing of receipt images using Tesseract
"""

import logging
import subprocess
import tempfile
from pathlib import Path
from typing import Optional
from decimal import Decimal

logger = logging.getLogger(__name__)


class OCRProcessor:
    """Process receipt images using Tesseract OCR"""
    
    def __init__(self):
        # Check if Tesseract is installed
        self.tesseract_available = self._check_tesseract()
        
    def _check_tesseract(self) -> bool:
        """Check if Tesseract is available"""
        try:
            result = subprocess.run(['tesseract', '--version'], capture_output=True, text=True)
            if result.returncode == 0:
                logger.info("Tesseract OCR is available")
                return True
        except:
            pass
        
        logger.warning("Tesseract OCR is not available. Install with: sudo apt-get install tesseract-ocr tesseract-ocr-rus")
        return False
    
    async def process_image(self, image_data: bytes) -> Optional[str]:
        """Process image and extract text using OCR"""
        if not self.tesseract_available:
            logger.error("Tesseract is not available")
            return None
        
        try:
            # Save image to temporary file
            with tempfile.NamedTemporaryFile(suffix='.png', delete=False) as tmp_file:
                tmp_file.write(image_data)
                tmp_path = tmp_file.name
            
            # Run Tesseract
            result = subprocess.run(
                ['tesseract', tmp_path, 'stdout', '-l', 'rus+eng', '--psm', '6'],
                capture_output=True,
                text=True
            )
            
            # Clean up
            Path(tmp_path).unlink()
            
            if result.returncode == 0:
                return result.stdout
            else:
                logger.error(f"Tesseract error: {result.stderr}")
                return None
                
        except Exception as e:
            logger.error(f"OCR processing error: {e}")
            return None
    
    def extract_amount_from_text(self, text: str) -> Optional[Decimal]:
        """Extract amount from OCR text"""
        import re
        
        # Look for amount patterns
        patterns = [
            r'([0-9\s]+(?:[,.]\d{2})?)\s*(?:₽|руб|RUB)',
            r'Сумма[:\s]+([0-9\s]+(?:[,.]\d{2})?)',
            r'Итого[:\s]+([0-9\s]+(?:[,.]\d{2})?)',
            r'Total[:\s]+([0-9\s]+(?:[,.]\d{2})?)',
        ]
        
        for pattern in patterns:
            matches = re.findall(pattern, text, re.IGNORECASE)
            for match in matches:
                try:
                    # Clean the amount string
                    amount_str = match.replace(' ', '').replace(',', '.')
                    amount = Decimal(amount_str)
                    if amount > 0:
                        return amount
                except:
                    continue
        
        return None