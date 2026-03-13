"use client";

import { useCallback, useEffect, useRef, useState } from "react";

import {
  demoDesigns,
  demoTemplates,
  type DemoDesign,
  type DemoTemplate,
} from "@/lib/demo-data";
import { DesignUploader } from "@/components/demo/design-uploader";
import { MockupResult } from "@/components/demo/mockup-result";
import { TemplateStrip } from "@/components/demo/template-strip";

interface DemoExperienceProps {
  compact?: boolean;
  demoReady?: boolean;
}

interface DemoResultState {
  mockupUrl: string;
  generationTimeMs: number;
  templateLabel: string;
}

export function DemoExperience({
  compact = false,
  demoReady = false,
}: DemoExperienceProps) {
  const [selectedDesign, setSelectedDesign] = useState<DemoDesign>(
    demoDesigns[0],
  );
  const [selectedTemplate, setSelectedTemplate] = useState<DemoTemplate>(
    demoTemplates[0],
  );
  const [designPreviewUrl, setDesignPreviewUrl] = useState(
    demoDesigns[0].previewSrc,
  );
  const [designPublicUrl, setDesignPublicUrl] = useState(
    demoDesigns[0].publicPath,
  );
  const [result, setResult] = useState<DemoResultState | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isGenerating, setIsGenerating] = useState(false);
  const [isUploading, setIsUploading] = useState(false);
  const objectUrlRef = useRef<string | null>(null);
  const didInitialRequest = useRef(false);
  const setupNotice =
    "Live demo preview is disabled locally until `MEETMOCKUP_API_KEY` and a public `NEXT_PUBLIC_SITE_URL` are configured.";

  const generate = useCallback(
    async (template: DemoTemplate, designUrl: string) => {
      if (!demoReady) {
        return;
      }

      setIsGenerating(true);
      setError(null);

      try {
        const response = await fetch("/api/demo/generate", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            template_id: template.id,
            design_url: designUrl,
            placement: {
              scale: 0.4,
              offset_x: 0,
              offset_y: -50,
            },
          }),
        });

        const data = await response.json();

        if (!response.ok) {
          setError(data.error ?? "Generation failed.");
          setResult(null);
          return;
        }

        setResult({
          mockupUrl: data.mockup_url,
          generationTimeMs: data.metadata?.generation_time_ms ?? 0,
          templateLabel: template.label,
        });
      } catch {
        setResult(null);
        setError("Something went wrong while contacting the demo API.");
      } finally {
        setIsGenerating(false);
      }
    },
    [demoReady],
  );

  useEffect(() => {
    if (didInitialRequest.current || !demoReady) return;

    didInitialRequest.current = true;
    void generate(demoTemplates[0], demoDesigns[0].publicPath);
  }, [demoReady, generate]);

  useEffect(() => {
    return () => {
      if (objectUrlRef.current) {
        URL.revokeObjectURL(objectUrlRef.current);
      }
    };
  }, []);

  const handleSampleSelect = (design: DemoDesign) => {
    setSelectedDesign(design);
    setDesignPreviewUrl(design.previewSrc);
    setDesignPublicUrl(design.publicPath);
    void generate(selectedTemplate, design.publicPath);
  };

  const handleFileSelect = async (file: File) => {
    if (objectUrlRef.current) {
      URL.revokeObjectURL(objectUrlRef.current);
    }

    const previewUrl = URL.createObjectURL(file);
    objectUrlRef.current = previewUrl;
    setSelectedDesign({
      id: "upload",
      label: file.name,
      previewSrc: previewUrl,
      publicPath: previewUrl,
    });
    setDesignPreviewUrl(previewUrl);
    setIsUploading(true);
    setError(null);

    try {
      // Try R2 upload first — returns a public URL the API can fetch
      const formData = new FormData();
      formData.append("file", file);

      const uploadRes = await fetch("/api/demo/upload", {
        method: "POST",
        body: formData,
      });
      const uploadData = await uploadRes.json();

      if (uploadRes.ok && uploadData.url) {
        setDesignPublicUrl(uploadData.url);
        await generate(selectedTemplate, uploadData.url);
        return;
      }

      // Fallback: convert to data URL if R2 is not configured
      const reader = new FileReader();
      const dataUrl = await new Promise<string>((resolve, reject) => {
        reader.onload = () => resolve(reader.result as string);
        reader.onerror = reject;
        reader.readAsDataURL(file);
      });

      setDesignPublicUrl(dataUrl);
      await generate(selectedTemplate, dataUrl);
    } catch {
      setError("Failed to process the uploaded file.");
    } finally {
      setIsUploading(false);
    }
  };

  const handleTemplateSelect = (template: DemoTemplate) => {
    setSelectedTemplate(template);
    void generate(template, designPublicUrl);
  };

  return (
    <div className="grid gap-5 lg:grid-cols-[0.85fr_1.15fr]">
      <div className="space-y-5">
        <DesignUploader
          currentPreviewUrl={designPreviewUrl}
          samples={demoDesigns}
          selectedDesignId={selectedDesign.id}
          isUploading={isUploading}
          onSampleSelect={handleSampleSelect}
          onFileSelect={handleFileSelect}
        />
        <TemplateStrip
          templates={demoTemplates}
          selectedTemplateId={selectedTemplate.id}
          onSelect={handleTemplateSelect}
        />
      </div>
      <MockupResult
        result={result}
        isGenerating={isGenerating}
        error={error}
        noticeMessage={!demoReady ? setupNotice : null}
        compact={compact}
      />
    </div>
  );
}
