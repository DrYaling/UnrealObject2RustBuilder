// Copyright Epic Games, Inc. All Rights Reserved.

#pragma once

#include "Elements/Interfaces/TypedElementObjectInterface.h"
#include "ComponentElementObjectInterface.generated.h"

UCLASS()
class ENGINE_API UComponentElementObjectInterface : public UTypedElementObjectInterface
{
	GENERATED_BODY()

public:
	virtual UObject* GetObject(const FTypedElementHandle& InElementHandle) override;
};
