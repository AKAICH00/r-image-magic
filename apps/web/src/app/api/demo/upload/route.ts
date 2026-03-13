import { randomUUID } from "node:crypto";

import { PutObjectCommand, S3Client } from "@aws-sdk/client-s3";
import { NextResponse } from "next/server";

export const runtime = "nodejs";

const MAX_UPLOAD_BYTES = 5 * 1024 * 1024;

function getR2Client(): S3Client | null {
  const accountId = process.env.R2_ACCOUNT_ID;
  const accessKeyId = process.env.R2_ACCESS_KEY_ID;
  const secretAccessKey = process.env.R2_SECRET_ACCESS_KEY;

  if (!accountId || !accessKeyId || !secretAccessKey) {
    return null;
  }

  return new S3Client({
    region: "auto",
    endpoint: `https://${accountId}.r2.cloudflarestorage.com`,
    credentials: { accessKeyId, secretAccessKey },
    forcePathStyle: true,
  });
}

export async function POST(request: Request) {
  const client = getR2Client();
  const bucket = process.env.R2_BUCKET_NAME ?? "meetmockup-demo-uploads";
  const publicUrlPrefix = process.env.R2_PUBLIC_URL_PREFIX;

  if (!client || !publicUrlPrefix) {
    return NextResponse.json(
      {
        error: "File uploads are not configured. Use a sample design instead.",
      },
      { status: 503 },
    );
  }

  const formData = await request.formData();
  const file = formData.get("file");

  if (!(file instanceof File)) {
    return NextResponse.json({ error: "No file uploaded." }, { status: 400 });
  }

  if (file.size > MAX_UPLOAD_BYTES) {
    return NextResponse.json(
      { error: "File too large. Use an image under 5 MB." },
      { status: 400 },
    );
  }

  const extension =
    file.type === "image/jpeg"
      ? "jpg"
      : file.type === "image/png"
        ? "png"
        : null;

  if (!extension) {
    return NextResponse.json(
      { error: "Only PNG and JPG uploads are supported." },
      { status: 400 },
    );
  }

  const key = `demo-uploads/${randomUUID()}.${extension}`;
  const buffer = Buffer.from(await file.arrayBuffer());

  await client.send(
    new PutObjectCommand({
      Bucket: bucket,
      Key: key,
      Body: buffer,
      ContentType: file.type,
    }),
  );

  const publicUrl = `${publicUrlPrefix.replace(/\/$/, "")}/${key}`;

  return NextResponse.json({ url: publicUrl });
}
