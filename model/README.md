# Bird Classification Model

## Overview

The Birdhouse Camera supports optional bird species classification by offloading
image analysis to an external server. This keeps the ESP32 firmware lightweight
and allows you to use state-of-the-art models without being constrained by the
microcontroller's limited resources.

## Recommended Setup

### Option 1: Local Classification Server (Recommended)

Run a lightweight Flask/FastAPI server on a Raspberry Pi or any machine on your
local network. The server receives JPEG images and returns species predictions.

**Expected API contract:**

```
POST /classify
Content-Type: image/jpeg
Body: <raw JPEG bytes>

Response 200:
{
  "species": "Northern Cardinal",
  "confidence": 0.94
}
```

**Example server using a pre-trained model:**

```python
# classify_server.py
from fastapi import FastAPI, Request
from fastapi.responses import JSONResponse
import torch
from torchvision import transforms, models
from PIL import Image
import io
import json

app = FastAPI()

# Load a fine-tuned bird classifier (e.g., from NABirds or CUB-200 dataset)
# Replace with your own trained model
model = models.mobilenet_v2(pretrained=False)
model.classifier[1] = torch.nn.Linear(model.last_channel, 400)  # 400 species
model.load_state_dict(torch.load("bird_classifier.pth", map_location="cpu"))
model.eval()

with open("species_labels.json") as f:
    labels = json.load(f)

transform = transforms.Compose([
    transforms.Resize((224, 224)),
    transforms.ToTensor(),
    transforms.Normalize(mean=[0.485, 0.456, 0.406],
                         std=[0.229, 0.224, 0.225]),
])

@app.post("/classify")
async def classify(request: Request):
    jpeg_data = await request.body()
    image = Image.open(io.BytesIO(jpeg_data)).convert("RGB")
    tensor = transform(image).unsqueeze(0)

    with torch.no_grad():
        output = model(tensor)
        prob = torch.softmax(output, dim=1)
        confidence, predicted = torch.max(prob, 1)

    species = labels[predicted.item()]
    return JSONResponse({
        "species": species,
        "confidence": round(confidence.item(), 3)
    })
```

Run with: `uvicorn classify_server:app --host 0.0.0.0 --port 8080`

Then set `CLASSIFICATION_SERVER=http://<server-ip>:8080` when building the firmware.

### Option 2: Home Assistant Add-on

You can also run the classification server as a Home Assistant add-on.
This keeps everything contained within your HA setup.

### Option 3: Cloud API

Use a cloud bird identification API (e.g., Merlin Bird ID API, iNaturalist).
Implement the adapter in `src/detection.rs` to match the expected response format.

## Training Your Own Model

For best results with your local bird population:

1. **Collect data**: Let the birdhouse camera run for a few weeks with detection-only mode
2. **Label images**: Use the captured snapshots to build a labeled dataset
3. **Fine-tune**: Start from a pre-trained MobileNetV2 or EfficientNet-Lite and fine-tune on your dataset
4. **Deploy**: Host the model on your local server

### Recommended Datasets for Pre-training

- [CUB-200-2011](http://www.vision.caltech.edu/datasets/cub_200_2011/) - 200 bird species
- [NABirds](https://dl.allninthings.net/nabirds/) - 400 North American bird species
- [iNaturalist](https://www.inaturalist.org/) - Community-sourced bird observations
