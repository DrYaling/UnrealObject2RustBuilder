// Copyright Epic Games, Inc. All Rights Reserved.

/*=============================================================================
	VolumetricCloudRendering.h
=============================================================================*/

#pragma once

#include "CoreMinimal.h"
#include "EngineDefines.h"
#include "RendererInterface.h"
#include "RenderResource.h"
#include "VolumeLighting.h"


class FScene;
class FViewInfo;
class FLightSceneInfo;
class UVolumetricCloudComponent;
class FVolumetricCloudSceneProxy;

struct FEngineShowFlags;



BEGIN_SHADER_PARAMETER_STRUCT(FVolumetricCloudCommonShaderParameters, )
	SHADER_PARAMETER(FLinearColor, GroundAlbedo)
	SHADER_PARAMETER(FVector, CloudLayerCenterKm)
	SHADER_PARAMETER(float, PlanetRadiusKm)
	SHADER_PARAMETER(float, BottomRadiusKm)
	SHADER_PARAMETER(float, TopRadiusKm)
	SHADER_PARAMETER(float, TracingStartMaxDistance)
	SHADER_PARAMETER(float, TracingMaxDistance)
	SHADER_PARAMETER(int32, SampleCountMin)
	SHADER_PARAMETER(int32, SampleCountMax)
	SHADER_PARAMETER(float, InvDistanceToSampleCountMax)
	SHADER_PARAMETER(int32, ShadowSampleCountMax)
	SHADER_PARAMETER(float, ShadowTracingMaxDistance)
	SHADER_PARAMETER(float, StopTracingTransmittanceThreshold)
	SHADER_PARAMETER(float, SkyLightCloudBottomVisibility)
	SHADER_PARAMETER_ARRAY(FLinearColor, AtmosphericLightCloudScatteredLuminanceScale, [2])
	SHADER_PARAMETER_ARRAY(float,	CloudShadowmapFarDepthKm, [2])
	SHADER_PARAMETER_ARRAY(float,	CloudShadowmapStrength, [2])
	SHADER_PARAMETER_ARRAY(float,	CloudShadowmapDepthBias, [2])
	SHADER_PARAMETER_ARRAY(float,	CloudShadowmapSampleCount, [2])
	SHADER_PARAMETER_ARRAY(FVector4,CloudShadowmapSizeInvSize, [2])
	SHADER_PARAMETER_ARRAY(FVector4,CloudShadowmapTracingSizeInvSize, [2])
	SHADER_PARAMETER_ARRAY(FMatrix,	CloudShadowmapWorldToLightClipMatrix, [2])
	SHADER_PARAMETER_ARRAY(FMatrix,	CloudShadowmapWorldToLightClipMatrixInv, [2])
	SHADER_PARAMETER_ARRAY(FVector4, CloudShadowmapTracingPixelScaleOffset, [2])
	SHADER_PARAMETER_ARRAY(FVector, CloudShadowmapLightDir, [2])
	SHADER_PARAMETER_ARRAY(FVector, CloudShadowmapLightPos, [2])
	SHADER_PARAMETER_ARRAY(FVector, CloudShadowmapLightAnchorPos, [2])	// Snapped position on the planet the shadow map rotate around 
	SHADER_PARAMETER(float,		CloudSkyAOFarDepthKm)
	SHADER_PARAMETER(float,		CloudSkyAOStrength)
	SHADER_PARAMETER(float,		CloudSkyAOSampleCount)
	SHADER_PARAMETER(FVector4,	CloudSkyAOSizeInvSize)
	SHADER_PARAMETER(FMatrix,	CloudSkyAOWorldToLightClipMatrix)
	SHADER_PARAMETER(FMatrix,	CloudSkyAOWorldToLightClipMatrixInv)
	SHADER_PARAMETER(FVector,	CloudSkyAOTraceDir)
END_SHADER_PARAMETER_STRUCT()

BEGIN_GLOBAL_SHADER_PARAMETER_STRUCT(FVolumetricCloudCommonGlobalShaderParameters, )
	SHADER_PARAMETER_STRUCT_INCLUDE(FVolumetricCloudCommonShaderParameters, VolumetricCloudCommonParams)
END_GLOBAL_SHADER_PARAMETER_STRUCT()

BEGIN_SHADER_PARAMETER_STRUCT(FLightCloudTransmittanceParameters, )
	SHADER_PARAMETER(FMatrix, CloudShadowmapWorldToLightClipMatrix)
	SHADER_PARAMETER(float, CloudShadowmapFarDepthKm)
	SHADER_PARAMETER_RDG_TEXTURE(Texture2D, CloudShadowmapTexture)
	SHADER_PARAMETER_SAMPLER(SamplerState, CloudShadowmapSampler)
	SHADER_PARAMETER(float, CloudShadowmapStrength)
END_SHADER_PARAMETER_STRUCT()

bool SetupLightCloudTransmittanceParameters(const FScene* Scene, const FViewInfo& View, const FLightSceneInfo* LightSceneInfo, FLightCloudTransmittanceParameters& OutParameters);

/** Contains render data created render side for a FVolumetricCloudSceneProxy objects. */
class FVolumetricCloudRenderSceneInfo
{
public:

	/** Initialization constructor. */
	explicit FVolumetricCloudRenderSceneInfo(FVolumetricCloudSceneProxy& VolumetricCloudSceneProxy);
	~FVolumetricCloudRenderSceneInfo();

	FVolumetricCloudSceneProxy& GetVolumetricCloudSceneProxy() const { return VolumetricCloudSceneProxy; }

	FVolumetricCloudCommonShaderParameters& GetVolumetricCloudCommonShaderParameters() { return VolumetricCloudCommonShaderParameters; }
	const FVolumetricCloudCommonShaderParameters& GetVolumetricCloudCommonShaderParameters() const { return VolumetricCloudCommonShaderParameters; }

	TUniformBufferRef<FVolumetricCloudCommonGlobalShaderParameters>& GetVolumetricCloudCommonShaderParametersUB() { return VolumetricCloudCommonShaderParametersUB; }
	const TUniformBufferRef<FVolumetricCloudCommonGlobalShaderParameters>& GetVolumetricCloudCommonShaderParametersUB() const { return VolumetricCloudCommonShaderParametersUB; }

private:

	FVolumetricCloudSceneProxy& VolumetricCloudSceneProxy;

	FVolumetricCloudCommonShaderParameters VolumetricCloudCommonShaderParameters;
	TUniformBufferRef<FVolumetricCloudCommonGlobalShaderParameters> VolumetricCloudCommonShaderParametersUB;
};


bool ShouldRenderVolumetricCloud(const FScene* Scene, const FEngineShowFlags& EngineShowFlags);
bool ShouldViewVisualizeVolumetricCloudConservativeDensity(const FViewInfo& ViewInfo, const FEngineShowFlags& EngineShowFlags);


// Structure with data necessary to specify a cloud render.
struct FCloudRenderContext
{
	///////////////////////////////////
	// Per scene parameters

	FVolumetricCloudRenderSceneInfo* CloudInfo;
	FMaterialRenderProxy* CloudVolumeMaterialProxy;

	FRDGTextureRef SceneDepthZ = nullptr;

	///////////////////////////////////
	// Per view parameters

	FViewInfo* MainView;
	TUniformBufferRef<FViewUniformShaderParameters> ViewUniformBuffer;
	FRenderTargetBindingSlots RenderTargets;

	bool bShouldViewRenderVolumetricRenderTarget;
	bool bSkipAerialPerspective;
	bool bIsReflectionRendering;				// Reflection capture and real time sky capture
	bool bIsSkyRealTimeReflectionRendering;		// Real time sky capture only
	bool bSkipAtmosphericLightShadowmap;
	bool bSecondAtmosphereLightEnabled;

	bool bAsyncCompute;
	bool bVisualizeConservativeDensity;

	FUintVector4 TracingCoordToZbufferCoordScaleBias;
	uint32 NoiseFrameIndexModPattern;

	FVolumeShadowingShaderParametersGlobal0 LightShadowShaderParams0;
	FRDGTextureRef VolumetricCloudShadowTexture[2];

	int VirtualShadowMapId0 = INDEX_NONE;

	FCloudRenderContext();

private:
};

BEGIN_SHADER_PARAMETER_STRUCT(FVolumetricCloudShadowAOParameters, )
	RDG_TEXTURE_ACCESS(ShadowMap0, ERHIAccess::SRVGraphics)
	RDG_TEXTURE_ACCESS(ShadowMap1, ERHIAccess::SRVGraphics)
	RDG_TEXTURE_ACCESS(SkyAO, ERHIAccess::SRVGraphics)
END_SHADER_PARAMETER_STRUCT()

FVolumetricCloudShadowAOParameters GetCloudShadowAOParameters(FRDGBuilder& GraphBuilder, const FViewInfo& View, const FVolumetricCloudRenderSceneInfo* CloudInfo);

struct FCloudShadowAOData
{
	bool bShouldSampleCloudShadow;
	bool bShouldSampleCloudSkyAO;
	FRDGTextureRef VolumetricCloudShadowMap[2];
	FRDGTextureRef VolumetricCloudSkyAO;
};

void GetCloudShadowAOData(const FVolumetricCloudRenderSceneInfo* CloudInfo, const FViewInfo& View, FRDGBuilder& GraphBuilder, FCloudShadowAOData& OutData);


